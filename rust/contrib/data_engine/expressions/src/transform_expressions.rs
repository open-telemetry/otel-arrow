// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TransformExpression {
    /// Remove data from a source and then write it to a destination.
    Move(MoveTransformExpression),

    /// Remove data from a target map.
    ReduceMap(ReduceMapTransformExpression),

    /// Remove data transformation.
    Remove(RemoveTransformExpression),

    /// Remove top-level keys from a target map.
    ///
    /// Note: Remove map keys is a specialized form of the reduce map
    /// transformation which only operates on top-level keys.
    RemoveMapKeys(RemoveMapKeysTransformExpression),

    /// Rename keys on a target map.
    RenameMapKeys(RenameMapKeysTransformExpression),

    /// Set data transformation.
    Set(SetTransformExpression),
}

impl TransformExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            TransformExpression::Move(m) => m.try_fold(scope),
            TransformExpression::ReduceMap(r) => r.try_fold(scope),
            TransformExpression::Remove(r) => r.try_fold(scope),
            TransformExpression::RemoveMapKeys(r) => r.try_fold(scope),
            TransformExpression::RenameMapKeys(r) => r.try_fold(scope),
            TransformExpression::Set(s) => s.try_fold(scope),
        }
    }
}

impl Expression for TransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TransformExpression::Move(m) => m.get_query_location(),
            TransformExpression::ReduceMap(r) => r.get_query_location(),
            TransformExpression::Remove(r) => r.get_query_location(),
            TransformExpression::RemoveMapKeys(r) => r.get_query_location(),
            TransformExpression::RenameMapKeys(r) => r.get_query_location(),
            TransformExpression::Set(s) => s.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TransformExpression::Move(_) => "Transform(Move)",
            TransformExpression::ReduceMap(r) => r.get_name(),
            TransformExpression::Remove(_) => "Transform(Remove)",
            TransformExpression::RemoveMapKeys(r) => r.get_name(),
            TransformExpression::RenameMapKeys(_) => "Transform(RenameMapKeys)",
            TransformExpression::Set(_) => "Transform(Set)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            TransformExpression::Move(m) => m.fmt_with_indent(f, indent),
            TransformExpression::ReduceMap(r) => r.fmt_with_indent(f, indent),
            TransformExpression::Remove(r) => r.fmt_with_indent(f, indent),
            TransformExpression::RemoveMapKeys(r) => r.fmt_with_indent(f, indent),
            TransformExpression::RenameMapKeys(r) => r.fmt_with_indent(f, indent),
            TransformExpression::Set(s) => s.fmt_with_indent(f, indent),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTransformExpression {
    query_location: QueryLocation,
    source: ScalarExpression,
    destination: MutableValueExpression,
}

impl SetTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        source: ScalarExpression,
        destination: MutableValueExpression,
    ) -> SetTransformExpression {
        Self {
            query_location,
            source,
            destination,
        }
    }

    pub fn get_source(&self) -> &ScalarExpression {
        &self.source
    }

    pub fn get_destination(&self) -> &MutableValueExpression {
        &self.destination
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.source.try_resolve_static(scope)?;
        self.destination.try_fold(scope)?;

        Ok(())
    }
}

impl Expression for SetTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "SetTransformExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Set")?;
        write!(f, "{indent}├── Source(Scalar): ")?;
        self.source
            .fmt_with_indent(f, format!("{indent}│                   ").as_str())?;
        write!(f, "{indent}└── Destination(Mutable): ")?;
        self.destination
            .fmt_with_indent(f, format!("{indent}                          ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
}

impl RemoveTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> RemoveTransformExpression {
        Self {
            query_location,
            target,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.target.try_fold(scope)?;

        Ok(())
    }
}

impl Expression for RemoveTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "RemoveTransformExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Remove")?;
        write!(f, "{indent}└── Target(Mutable): ")?;
        self.target
            .fmt_with_indent(f, format!("{indent}                     ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveTransformExpression {
    query_location: QueryLocation,
    source: MutableValueExpression,
    destination: MutableValueExpression,
}

impl MoveTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        source: MutableValueExpression,
        destination: MutableValueExpression,
    ) -> MoveTransformExpression {
        Self {
            query_location,
            source,
            destination,
        }
    }

    pub fn get_source(&self) -> &MutableValueExpression {
        &self.source
    }

    pub fn get_destination(&self) -> &MutableValueExpression {
        &self.destination
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.source.try_fold(scope)?;
        self.destination.try_fold(scope)?;

        Ok(())
    }
}

impl Expression for MoveTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MoveTransformExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Move")?;
        write!(f, "{indent}├── Source(Mutable): ")?;
        self.source
            .fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
        write!(f, "{indent}└── Destination(Mutable): ")?;
        self.destination
            .fmt_with_indent(f, format!("{indent}                          ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveMapKeysTransformExpression {
    /// A map key transformation providing the top-level keys to remove. All other data is retained.
    Remove(MapKeyListExpression),

    /// A map key transformation providing the top-level keys to retain. All other data is removed.
    Retain(MapKeyListExpression),
}

impl RemoveMapKeysTransformExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            RemoveMapKeysTransformExpression::Remove(m)
            | RemoveMapKeysTransformExpression::Retain(m) => m.try_fold(scope),
        }
    }
}

impl Expression for RemoveMapKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            RemoveMapKeysTransformExpression::Remove(m) => m.get_query_location(),
            RemoveMapKeysTransformExpression::Retain(m) => m.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            RemoveMapKeysTransformExpression::Remove(_) => "RemoveMapKeysTransform(Remove)",
            RemoveMapKeysTransformExpression::Retain(_) => "RemoveMapKeysTransform(Retain)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            RemoveMapKeysTransformExpression::Remove(m) => {
                writeln!(f, "RemoveMapKeys(Remove)")?;
                m.fmt_with_indent(f, indent)
            }
            RemoveMapKeysTransformExpression::Retain(m) => {
                writeln!(f, "RemoveMapKeys(Retain)")?;
                m.fmt_with_indent(f, indent)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapKeyListExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys: Vec<ScalarExpression>,
}

impl MapKeyListExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys: Vec<ScalarExpression>,
    ) -> MapKeyListExpression {
        Self {
            query_location,
            target,
            keys,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_keys(&self) -> &[ScalarExpression] {
        &self.keys
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.target.try_fold(scope)?;

        for k in &mut self.keys {
            k.try_resolve_static(scope)?;
        }

        Ok(())
    }
}

impl Expression for MapKeyListExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapKeyListExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        write!(f, "{indent}├── Target(Mutable): ")?;
        self.target
            .fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
        if self.keys.is_empty() {
            writeln!(f, "{indent}└── Keys: []")?;
        } else {
            let last_idx = self.keys.len() - 1;
            for (i, k) in self.keys.iter().enumerate() {
                if i == last_idx {
                    write!(f, "{indent}└── Keys[{i}](Scalar): ")?;
                    k.fmt_with_indent(f, format!("{indent}                     ").as_str())?;
                } else {
                    write!(f, "{indent}├── Keys[{i}](Scalar): ")?;
                    k.fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReduceMapTransformExpression {
    /// A map reduction providing the data to remove. All other data is retained.
    Remove(MapSelectionExpression),

    /// A map reduction providing the data to retain. All other data is removed.
    Retain(MapSelectionExpression),
}

impl ReduceMapTransformExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            ReduceMapTransformExpression::Remove(m) | ReduceMapTransformExpression::Retain(m) => {
                m.try_fold(scope)
            }
        }
    }
}

impl Expression for ReduceMapTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ReduceMapTransformExpression::Remove(m) => m.get_query_location(),
            ReduceMapTransformExpression::Retain(m) => m.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ReduceMapTransformExpression::Remove(_) => "ReduceMapTransform(Remove)",
            ReduceMapTransformExpression::Retain(_) => "ReduceMapTransform(Retain)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            ReduceMapTransformExpression::Remove(m) => {
                writeln!(f, "ReduceMap(Remove)")?;
                m.fmt_with_indent(f, indent)
            }
            ReduceMapTransformExpression::Retain(m) => {
                writeln!(f, "ReduceMap(Retain)")?;
                m.fmt_with_indent(f, indent)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapSelector {
    /// A top-level key or key pattern.
    KeyOrKeyPattern(ScalarExpression),

    /// A [`ValueAccessor`] representing a path to data on the map.
    ///
    /// Note: The [`ValueAccessor`] could refer to top-level keys or nested
    /// elements.
    ValueAccessor(ValueAccessor),
}

impl MapSelector {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            MapSelector::KeyOrKeyPattern(k) => {
                k.try_resolve_static(scope)?;
                Ok(())
            }
            MapSelector::ValueAccessor(v) => {
                v.try_fold(scope)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSelectionExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    selectors: Vec<MapSelector>,
}

impl MapSelectionExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> MapSelectionExpression {
        Self {
            query_location,
            target,
            selectors: Vec::new(),
        }
    }

    pub fn new_with_selectors(
        query_location: QueryLocation,
        target: MutableValueExpression,
        selectors: Vec<MapSelector>,
    ) -> MapSelectionExpression {
        Self {
            query_location,
            target,
            selectors,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_selectors(&self) -> &[MapSelector] {
        &self.selectors
    }

    pub fn push_key_or_key_pattern(&mut self, key_or_key_pattern: ScalarExpression) -> bool {
        if let ScalarExpression::Static(s) = &key_or_key_pattern {
            let value_type = s.get_value_type();
            if value_type != ValueType::String && value_type != ValueType::Regex {
                return false;
            }
        }

        self.selectors
            .push(MapSelector::KeyOrKeyPattern(key_or_key_pattern));

        true
    }

    pub fn push_value_accessor(&mut self, value_accessor: ValueAccessor) -> bool {
        assert!(value_accessor.has_selectors());

        if let ScalarExpression::Static(s) = value_accessor.get_selectors().first().unwrap()
            && s.get_value_type() != ValueType::String
        {
            return false;
        }

        self.selectors
            .push(MapSelector::ValueAccessor(value_accessor));

        true
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.target.try_fold(scope)?;

        for s in &mut self.selectors {
            s.try_fold(scope)?;
        }

        Ok(())
    }
}

impl Expression for MapSelectionExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapSelectionExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        write!(f, "{indent}├── Target(Mutable): ")?;
        self.target
            .fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
        if self.selectors.is_empty() {
            writeln!(f, "{indent}└── Selectors: []")?;
        } else {
            for (i, sel) in self.selectors.iter().enumerate() {
                let last = i + 1 == self.selectors.len();
                let branch = if last { "└──" } else { "├──" };
                match sel {
                    MapSelector::KeyOrKeyPattern(s) => {
                        write!(f, "{indent}{branch} Selectors[{i}](KeyOrPattern): ")?;
                        s.fmt_with_indent(
                            f,
                            format!(
                                "{indent}{}                               ",
                                if last { " " } else { "│" }
                            )
                            .as_str(),
                        )?;
                    }
                    MapSelector::ValueAccessor(a) => {
                        write!(f, "{indent}{branch} Selectors[{i}](Accessor): ")?;
                        a.fmt_with_indent(
                            f,
                            format!("{indent}{}   ", if last { " " } else { "│" }).as_str(),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenameMapKeysTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys: Vec<MapKeyRenameSelector>,
}

impl RenameMapKeysTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys: Vec<MapKeyRenameSelector>,
    ) -> RenameMapKeysTransformExpression {
        Self {
            query_location,
            target,
            keys,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_keys(&self) -> &[MapKeyRenameSelector] {
        &self.keys
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.target.try_fold(scope)?;

        for k in &mut self.keys {
            k.try_fold(scope)?;
        }

        Ok(())
    }
}

impl Expression for RenameMapKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "RenameMapKeysTransformExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "RenameMapKeys")?;
        write!(f, "{indent}├── Target(Mutable): ")?;
        self.target
            .fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
        if self.keys.is_empty() {
            writeln!(f, "{indent}└── Keys: []")?;
        } else {
            let last_idx = self.keys.len() - 1;
            for (i, k) in self.keys.iter().enumerate() {
                if i == last_idx {
                    writeln!(f, "{indent}└── Keys[{i}]:")?;
                    k.fmt_with_indent(f, format!("{indent}    ").as_str())?;
                } else {
                    writeln!(f, "{indent}├── Keys[{i}]:")?;
                    k.fmt_with_indent(f, format!("{indent}│   ").as_str())?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapKeyRenameSelector {
    source: ValueAccessor,
    destination: ValueAccessor,
}

impl MapKeyRenameSelector {
    pub fn new(source: ValueAccessor, destination: ValueAccessor) -> MapKeyRenameSelector {
        Self {
            source,
            destination,
        }
    }

    pub fn get_source(&self) -> &ValueAccessor {
        &self.source
    }

    pub fn get_destination(&self) -> &ValueAccessor {
        &self.destination
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        self.source.try_fold(scope)?;
        self.destination.try_fold(scope)?;
        Ok(())
    }

    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        write!(f, "{indent}├── Source(Accessor): ")?;
        self.source
            .fmt_with_indent(f, format!("{indent}│   ").as_str())?;
        write!(f, "{indent}└── Destination(Accessor):")?;
        self.destination
            .fmt_with_indent(f, format!("{indent}    ").as_str())?;
        Ok(())
    }
}
