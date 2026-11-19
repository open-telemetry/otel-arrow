// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded query-scope dependencies for authored incremental pagination.

use super::{DatabaseSystem, SourceQueryError, has_rownum_filter, parameter, same_ident};
use sqlparser::ast::{
    Expr, Ident, JoinOperator, Query, Select, SetExpr, TableFactor, TableWithJoins, Value,
    ValueWithSpan, Visit, Visitor,
};
use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
    ptr,
};

#[derive(Default)]
struct Node {
    inputs: Vec<usize>,
    cursor: bool,
    pagination: bool,
    isolation_boundary: bool,
}

struct Binding {
    name: Ident,
    node: usize,
}

struct Scope {
    node: usize,
    ctes: Vec<Binding>,
    recursive: bool,
    definition: Option<usize>,
    sources: Option<usize>,
    correlation_floor: usize,
    single_select: bool,
}

struct Dependencies<'a> {
    system: DatabaseSystem,
    binds: &'a [String],
    nodes: Vec<Node>,
    // Addresses identify nodes only during this immutable AST traversal.
    queries: HashMap<*const Query, usize>,
    relations: HashMap<*const TableFactor, usize>,
    apply: HashSet<*const TableFactor>,
    blocks: Vec<usize>,
    scopes: Vec<Scope>,
}

impl Dependencies<'_> {
    fn new_node(&mut self) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::default());
        index
    }

    fn query_node(&mut self, query: &Query) -> usize {
        let address = ptr::from_ref(query);
        if let Some(&node) = self.queries.get(&address) {
            return node;
        }
        let node = self.new_node();
        let _ = self.queries.insert(address, node);
        node
    }

    fn input(&mut self, input: usize) -> ControlFlow<SourceQueryError> {
        let Some(&node) = self.blocks.last() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        self.nodes[node].inputs.push(input);
        ControlFlow::Continue(())
    }

    fn mark_cursor(&mut self) -> ControlFlow<SourceQueryError> {
        let Some(&node) = self.blocks.last() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        self.nodes[node].cursor = true;
        ControlFlow::Continue(())
    }

    fn mark_apply(&mut self, from: &TableWithJoins) {
        for join in &from.joins {
            if matches!(
                join.join_operator,
                JoinOperator::CrossApply | JoinOperator::OuterApply
            ) {
                let _ = self.apply.insert(ptr::from_ref(&join.relation));
            }
        }
    }

    fn cte_inputs(&self, name: &Ident) -> Vec<usize> {
        let mut inputs = Vec::new();
        for scope in self.scopes.iter().rev() {
            let visible = match scope.definition {
                Some(index) if !scope.recursive => {
                    index + usize::from(self.system != DatabaseSystem::PostgreSQL)
                }
                _ => scope.ctes.len(),
            };
            let mut shadows = false;
            for binding in &scope.ctes[..visible] {
                let (possible, definite) = cte_name_match(name, &binding.name, self.system);
                if possible {
                    inputs.push(binding.node);
                }
                shadows |= definite;
            }
            if shadows {
                break;
            }
        }
        inputs
    }

    fn table_commands(&mut self, body: &SetExpr) -> ControlFlow<SourceQueryError> {
        let mut pending = vec![body];
        while let Some(body) = pending.pop() {
            match body {
                SetExpr::SetOperation { left, right, .. } => {
                    pending.extend([left.as_ref(), right.as_ref()]);
                }
                SetExpr::Table(table) if table.schema_name.is_none() => {
                    // sqlparser 0.63 consumes the next two tokens after an
                    // unqualified TABLE name, hiding LIMIT and even its bind.
                    return ControlFlow::Break(SourceQueryError::Composition);
                }
                _ => {}
            }
        }
        ControlFlow::Continue(())
    }
}

// Dependency checks must not assume a SQL Server collation is case-sensitive.
// Uncertain matches retain outer candidates rather than hiding a dependency.
fn cte_name_match(left: &Ident, right: &Ident, system: DatabaseSystem) -> (bool, bool) {
    if system == DatabaseSystem::SqlServer {
        let left = left.value.trim_end_matches(' ');
        let right = right.value.trim_end_matches(' ');
        return (
            !left.is_ascii() || !right.is_ascii() || left.eq_ignore_ascii_case(right),
            left == right,
        );
    }
    if left.value.is_ascii() && right.value.is_ascii() {
        let equal = same_ident(left, right, system);
        return (equal, equal);
    }
    if left.quote_style.is_some() && right.quote_style.is_some() {
        let equal = left.value == right.value;
        return (equal, equal);
    }
    (
        true,
        left.quote_style.is_some() == right.quote_style.is_some() && left.value == right.value,
    )
}

impl Visitor for Dependencies<'_> {
    type Break = SourceQueryError;

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        let node = self.query_node(query);
        let correlation_floor = if self
            .blocks
            .last()
            .is_some_and(|&parent| self.nodes[parent].isolation_boundary)
        {
            self.scopes.len()
        } else {
            self.scopes
                .last()
                .map_or(0, |scope| scope.correlation_floor)
        };
        // Correlated scopes can read cursor-derived values from outer inputs.
        // Do not inherit outer WHERE/projection dependencies: the mandatory
        // outer guard alone must not make an independent source unsafe.
        for source in self.scopes[correlation_floor..]
            .iter()
            .filter_map(|scope| scope.sources)
        {
            self.nodes[node].inputs.push(source);
        }
        let definition = if let Some(parent) = self.scopes.last_mut() {
            let definition = parent.ctes.iter().position(|binding| binding.node == node);
            if definition.is_some() {
                parent.definition = definition;
            }
            definition.is_some()
        } else {
            false
        };
        if !definition && !self.blocks.is_empty() {
            self.input(node)?;
        }
        self.nodes[node].pagination = query.limit_clause.is_some() || query.fetch.is_some();
        let mut ctes = Vec::new();
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                ctes.push(Binding {
                    name: cte.alias.name.clone(),
                    node: self.query_node(&cte.query),
                });
            }
        }
        self.scopes.push(Scope {
            node,
            ctes,
            recursive: query.with.as_ref().is_some_and(|with| with.recursive),
            definition: None,
            sources: None,
            correlation_floor,
            single_select: matches!(&*query.body, SetExpr::Select(_)),
        });
        self.blocks.push(node);
        self.table_commands(&query.body)?;
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<Self::Break> {
        let Some(scope) = self.scopes.pop() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        if self.blocks.pop() != Some(scope.node) {
            return ControlFlow::Break(SourceQueryError::Composition);
        }
        if let Some(parent) = self.scopes.last_mut()
            && parent
                .definition
                .is_some_and(|index| parent.ctes[index].node == scope.node)
        {
            parent.definition = None;
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<Self::Break> {
        let node = self.new_node();
        let sources = self.new_node();
        self.nodes[node].inputs.push(sources);
        let Some(scope) = self.scopes.last() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        for source in self.scopes[scope.correlation_floor..self.scopes.len() - 1]
            .iter()
            .filter_map(|scope| scope.sources)
        {
            self.nodes[node].inputs.push(source);
        }
        let Some(scope) = self.scopes.last_mut() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        scope.sources = Some(sources);
        for from in &select.from {
            self.mark_apply(from);
        }
        self.nodes[node].pagination = select.top.is_some()
            || (self.system == DatabaseSystem::Oracle && has_rownum_filter(select));
        self.input(node)?;
        self.blocks.push(node);
        ControlFlow::Continue(())
    }

    fn post_visit_select(&mut self, _: &Select) -> ControlFlow<Self::Break> {
        if self.blocks.pop().is_none() {
            return ControlFlow::Break(SourceQueryError::Composition);
        }
        let Some(scope) = self.scopes.last_mut() else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        if !scope.single_select {
            scope.sources = None;
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<Self::Break> {
        let node = self.new_node();
        let address = ptr::from_ref(table);
        let _ = self.relations.insert(address, node);
        self.nodes[node].isolation_boundary =
            matches!(table, TableFactor::Derived { lateral: false, .. })
                && !self.apply.contains(&address);
        self.input(node)?;
        let Some(source) = self.scopes.last().and_then(|scope| scope.sources) else {
            return ControlFlow::Break(SourceQueryError::Composition);
        };
        self.nodes[source].inputs.push(node);
        self.blocks.push(node);
        if let TableFactor::NestedJoin {
            table_with_joins, ..
        } = table
        {
            self.mark_apply(table_with_joins);
        }
        if let TableFactor::Table { name, args, .. } = table
            && (args.is_none() || self.system == DatabaseSystem::SqlServer)
            && let [part] = name.0.as_slice()
            && let Some(name) = part.as_ident()
        {
            for input in self.cte_inputs(name) {
                self.input(input)?;
            }
        }
        ControlFlow::Continue(())
    }

    fn post_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<Self::Break> {
        if self.blocks.pop() != self.relations.get(&ptr::from_ref(table)).copied() {
            return ControlFlow::Break(SourceQueryError::Composition);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        if self.binds.iter().any(|bind| parameter(expr, bind)) {
            self.mark_cursor()?;
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_value(&mut self, value: &ValueWithSpan) -> ControlFlow<Self::Break> {
        if let Value::Placeholder(name) = &value.value
            && self.binds.contains(name)
        {
            self.mark_cursor()?;
        }
        ControlFlow::Continue(())
    }
}

pub(super) fn validate(
    query: &Query,
    binds: &[String],
    system: DatabaseSystem,
) -> Result<(), SourceQueryError> {
    let mut dependencies = Dependencies {
        system,
        binds,
        nodes: Vec::new(),
        queries: HashMap::new(),
        relations: HashMap::new(),
        apply: HashSet::new(),
        blocks: Vec::new(),
        scopes: Vec::new(),
    };
    let root = dependencies.query_node(query);
    if let ControlFlow::Break(error) = query.visit(&mut dependencies) {
        return Err(error);
    }
    let nodes = dependencies.nodes;
    let mut users = vec![Vec::new(); nodes.len()];
    let mut dependent: Vec<_> = nodes.iter().map(|node| node.cursor).collect();
    let mut pending = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        for &input in &node.inputs {
            users[input].push(index);
        }
        if node.cursor {
            pending.push(index);
        }
    }
    while let Some(input) = pending.pop() {
        for &user in &users[input] {
            if !dependent[user] {
                dependent[user] = true;
                pending.push(user);
            }
        }
    }
    let mut reachable = vec![false; nodes.len()];
    pending.push(root);
    while let Some(node) = pending.pop() {
        if reachable[node] {
            continue;
        }
        reachable[node] = true;
        if nodes[node].pagination && dependent[node] {
            return Err(SourceQueryError::Pagination);
        }
        pending.extend_from_slice(&nodes[node].inputs);
    }
    Ok(())
}
