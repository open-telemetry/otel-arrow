//! Receiver-local XML document view backed by quick-xml's namespace-aware reader.
//!
//! Provides the read-only traversal needed by SOAP, event, query, and bookmark
//! handling without a second XML parser dependency. This is not a schema validator
//! or a mutable DOM: protocol-specific structure and scalar-field requirements
//! belong to callers.
//!
//! # Representation and limits
//!
//! Documents borrow the original UTF-8 input and own decoded names, attributes,
//! and text in a flat, preorder arena. Parent/child/sibling links are indices;
//! parsing uses an explicit stack and dropping the document does not recursively
//! drop a tree. Node handles borrow the arena, while source fragments borrow the
//! original input independently.
//!
//! The node budget counts elements, merged text nodes, and comment/processing
//! instruction placeholders inside the root. Attributes and namespace declarations
//! are stored on elements and do not count as nodes. A separate parser-wide byte
//! budget, eight times the input length, charges copied strings and every resolved
//! namespace use before decoding. Namespace strings are interned, but repeated
//! expansion still consumes budget to bound processing work. Arena and reader
//! overhead are additional; callers must bound input size separately. There is no
//! independent nesting-depth limit in this layer.
//!
//! # Parsing policy
//!
//! Parsing requires one closed root element and rejects DTDs, unknown entities,
//! prohibited XML characters, undeclared prefixes, duplicate expanded attribute
//! names, and invalid reserved namespace bindings. Optional declarations must be
//! XML 1.0 and appear at the start. No external resources are loaded. Input must
//! already be decoded to UTF-8; declaration encoding names are syntax-checked but
//! do not select a transcoder or validate the original transport encoding.
//!
//! # Decoded values versus source
//!
//! Text and CDATA normalize literal CR/CRLF to LF. Attribute values normalize
//! literal XML whitespace to spaces before entity expansion, preserving whitespace
//! explicitly supplied through character references. Adjacent text, CDATA, and
//! references merge into one text node; comments and processing instructions
//! interrupt merging. Their contents are not exposed as decoded text.
//!
//! Expanded names use namespace URIs rather than prefix spellings. Default
//! namespaces apply to elements, not unprefixed attributes. Namespace declarations
//! are separate from ordinary attributes. Original source spans retain markup and
//! escaping; they do not automatically include inherited namespace declarations.
//! Bookmark serialization uses `Node::namespaces` to make detached XML reusable.

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use std::{collections::HashMap, ops::Range, rc::Rc};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

/// XML reader failures or receiver-local well-formedness and policy violations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Syntax, decoding, attribute, or entity errors supplied by quick-xml.
    #[error("invalid XML: {0}")]
    Reader(#[from] quick_xml::Error),
    /// An additional document, namespace, character, or capacity check failed.
    #[error("invalid XML: {0}")]
    Invalid(&'static str),
}

/// Expanded name with an immutable shared namespace URI and an owned local name.
#[derive(Default)]
pub struct Name {
    namespace: Option<Rc<str>>,
    local: String,
}

impl Name {
    /// Local name without its namespace prefix.
    pub fn name(&self) -> &str {
        &self.local
    }

    /// Resolved namespace URI, or `None` for an unqualified name.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

/// Ordinary element attribute with an expanded name and normalized decoded value.
/// Namespace declarations are represented separately by `Namespace`.
pub struct Attribute {
    name: Name,
    value: String,
}

impl Attribute {
    /// Local attribute name, excluding any prefix.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Resolved namespace; unprefixed attributes do not inherit a default namespace.
    pub fn namespace(&self) -> Option<&str> {
        self.name.namespace()
    }

    /// Value after XML whitespace normalization and entity expansion.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Explicit namespace declaration stored on an element.
pub struct Namespace {
    prefix: Option<String>,
    uri: String,
}

impl Namespace {
    /// Declared prefix, or `None` for the default namespace.
    pub fn name(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Decoded namespace URI; an empty default URI cancels inherited qualification.
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

/// Traversal content, with placeholders retaining non-text sibling boundaries.
enum Content {
    Element {
        name: Name,
        attributes: Vec<Attribute>,
        namespaces: Vec<Namespace>,
    },
    Text(String),
    Other,
}

/// Arena entry whose subtree occupies `index..subtree_end` after parsing.
/// Source ranges use byte offsets into the borrowed input, not decoded values.
struct Entry {
    content: Content,
    parent: Option<usize>,
    first_child: Option<usize>,
    last_child: Option<usize>,
    next_sibling: Option<usize>,
    subtree_end: usize,
    range: Range<usize>,
}

/// Parsed document owning traversal data and borrowing original XML source text.
pub struct Document<'input> {
    input: &'input str,
    entries: Vec<Entry>,
    root: usize,
    empty_name: Name,
}

/// Accounts for string storage and repeated namespace expansion before allocation.
/// Immutable namespaces share storage with non-atomic, thread-local ownership.
struct ParseBudget {
    remaining_bytes: usize,
    namespaces: HashMap<String, Rc<str>>,
}

impl ParseBudget {
    fn new(input_bytes: usize) -> Self {
        Self {
            remaining_bytes: input_bytes.saturating_mul(8),
            namespaces: HashMap::new(),
        }
    }

    fn charge(&mut self, bytes: usize) -> Result<(), Error> {
        self.remaining_bytes = self
            .remaining_bytes
            .checked_sub(bytes)
            .ok_or(Error::Invalid("XML expanded byte budget exceeded"))?;
        Ok(())
    }

    fn namespace(&mut self, raw: &str) -> Result<Rc<str>, Error> {
        self.charge(raw.len())?;
        if let Some(namespace) = self.namespaces.get(raw) {
            return Ok(Rc::clone(namespace));
        }
        self.charge(raw.len())?;
        let namespace: Rc<str> = attribute_value(raw)?.into();
        let _ = self
            .namespaces
            .insert(raw.to_owned(), Rc::clone(&namespace));
        Ok(namespace)
    }
}

impl<'input> Document<'input> {
    /// Parse using the input byte length as the maximum number of arena nodes.
    /// This convenience limit does not impose an input-size policy on callers.
    pub fn parse(input: &'input str) -> Result<Self, Error> {
        Self::parse_bounded(input, input.len())
    }

    /// Parse a complete XML 1.0 document with an explicit arena-node budget.
    ///
    /// Returns no partial document on error. Outside the root, only XML whitespace,
    /// comments, processing instructions, and a permitted declaration are accepted.
    /// `max_nodes == 0` rejects any root. Element attributes are decoded before that
    /// element's node-budget check, but are charged to the expanded byte budget
    /// first. The budgets are not an exact allocator-memory ceiling.
    pub fn parse_bounded(input: &'input str, max_nodes: usize) -> Result<Self, Error> {
        if !input.chars().all(xml_character) {
            return Err(Error::Invalid("prohibited character"));
        }
        let mut reader = NsReader::from_str(input);
        reader.config_mut().check_comments = true;
        let mut budget = ParseBudget::new(input.len());
        let mut document = Self {
            input,
            entries: Vec::new(),
            root: 0,
            empty_name: Name::default(),
        };
        let mut stack = Vec::new();
        let mut root_seen = false;
        let mut declaration_allowed = true;
        loop {
            let start = reader.buffer_position() as usize;
            let event = reader.read_event()?;
            let end = reader.buffer_position() as usize;
            let parent = stack.last().copied();
            match event {
                Event::Start(ref element) | Event::Empty(ref element) => {
                    if parent.is_none() && root_seen {
                        return Err(Error::Invalid("multiple root elements"));
                    }
                    let name = resolve_name(&reader, element.name(), false, &mut budget)?;
                    let mut attributes = Vec::new();
                    let mut namespaces = Vec::new();
                    for attribute in element.attributes() {
                        let attribute = attribute.map_err(quick_xml::Error::from)?;
                        let key = std::str::from_utf8(attribute.key.as_ref())
                            .map_err(|_| Error::Invalid("attribute name encoding"))?;
                        validate_name(key)?;
                        let raw = std::str::from_utf8(&attribute.value)
                            .map_err(|_| Error::Invalid("attribute encoding"))?;
                        if raw.contains('<') {
                            return Err(Error::Invalid("unescaped attribute delimiter"));
                        }
                        budget.charge(key.len().saturating_add(raw.len()))?;
                        let value = attribute_value(raw)?;
                        if key == "xmlns" || key.starts_with("xmlns:") {
                            let prefix = key.strip_prefix("xmlns:");
                            if prefix == Some("xmlns")
                                || value == XMLNS_NAMESPACE
                                || (prefix == Some("xml")) != (value == XML_NAMESPACE)
                                || (prefix.is_some() && value.is_empty())
                            {
                                return Err(Error::Invalid("reserved or empty namespace binding"));
                            }
                            namespaces.push(Namespace {
                                prefix: prefix.map(str::to_owned),
                                uri: value,
                            });
                        } else {
                            let name = resolve_name(&reader, attribute.key, true, &mut budget)?;
                            if attributes.iter().any(|previous: &Attribute| {
                                previous.name.local == name.local
                                    && previous.name.namespace == name.namespace
                            }) {
                                return Err(Error::Invalid("duplicate expanded attribute"));
                            }
                            attributes.push(Attribute { name, value });
                        }
                    }
                    let index = document.push(
                        Content::Element {
                            name,
                            attributes,
                            namespaces,
                        },
                        parent,
                        start..end,
                        max_nodes,
                    )?;
                    if parent.is_none() {
                        document.root = index;
                        root_seen = true;
                    }
                    if matches!(event, Event::Start(_)) {
                        stack.push(index);
                    }
                }
                Event::End(_) => {
                    let index = stack
                        .pop()
                        .ok_or(Error::Invalid("unexpected closing tag"))?;
                    document.entries[index].range.end = end;
                    document.entries[index].subtree_end = document.entries.len();
                }
                Event::Text(text) => {
                    budget.charge(text.len())?;
                    let value = text.xml10_content().map_err(quick_xml::Error::from)?;
                    if value.contains("]]>") {
                        return Err(Error::Invalid("CDATA delimiter in text"));
                    }
                    document.push_text(&value, parent, start..end, max_nodes)?;
                }
                Event::CData(text) => {
                    if parent.is_none() {
                        return Err(Error::Invalid("CDATA outside root"));
                    }
                    budget.charge(text.len())?;
                    let value = text.xml10_content().map_err(quick_xml::Error::from)?;
                    document.push_text(&value, parent, start..end, max_nodes)?;
                }
                Event::GeneralRef(reference) => {
                    if parent.is_none() {
                        return Err(Error::Invalid("entity outside root"));
                    }
                    budget.charge(reference.len().saturating_add(2))?;
                    let name = reference.decode().map_err(quick_xml::Error::from)?;
                    let escaped = format!("&{name};");
                    let value =
                        quick_xml::escape::unescape(&escaped).map_err(quick_xml::Error::from)?;
                    if !value.chars().all(xml_character) {
                        return Err(Error::Invalid("prohibited character reference"));
                    }
                    document.push_text(&value, parent, start..end, max_nodes)?;
                }
                Event::DocType(_) => return Err(Error::Invalid("DTDs are prohibited")),
                Event::Decl(declaration) => {
                    if !declaration_allowed {
                        return Err(Error::Invalid("unexpected XML declaration"));
                    }
                    validate_declaration(&declaration)?;
                }
                Event::PI(instruction) => {
                    let target = std::str::from_utf8(instruction.target())
                        .map_err(|_| Error::Invalid("processing instruction encoding"))?;
                    validate_name(target)?;
                    if target.eq_ignore_ascii_case("xml") {
                        return Err(Error::Invalid("reserved processing instruction target"));
                    }
                    if parent.is_some() {
                        let _ = document.push(Content::Other, parent, start..end, max_nodes)?;
                    }
                }
                Event::Comment(_) => {
                    if parent.is_some() {
                        let _ = document.push(Content::Other, parent, start..end, max_nodes)?;
                    }
                }
                Event::Eof => break,
            }
            declaration_allowed = false;
        }
        if !root_seen || !stack.is_empty() {
            return Err(Error::Invalid("missing or unclosed root"));
        }
        Ok(document)
    }

    /// Append one budgeted entry and link it as its parent's last direct child.
    fn push(
        &mut self,
        content: Content,
        parent: Option<usize>,
        range: Range<usize>,
        max_nodes: usize,
    ) -> Result<usize, Error> {
        if self.entries.len() >= max_nodes {
            return Err(Error::Invalid("node limit exceeded"));
        }
        let index = self.entries.len();
        self.entries.push(Entry {
            content,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            subtree_end: index + 1,
            range,
        });
        if let Some(parent) = parent {
            if let Some(previous) = self.entries[parent].last_child {
                self.entries[previous].next_sibling = Some(index);
            } else {
                self.entries[parent].first_child = Some(index);
            }
            self.entries[parent].last_child = Some(index);
        }
        Ok(index)
    }

    /// Merge adjacent decoded text or allocate a node; ignore whitespace outside root.
    /// The merged node's source range spans all contributing source fragments.
    fn push_text(
        &mut self,
        value: &str,
        parent: Option<usize>,
        range: Range<usize>,
        max_nodes: usize,
    ) -> Result<(), Error> {
        let Some(parent) = parent else {
            return if value
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
            {
                Ok(())
            } else {
                Err(Error::Invalid("text outside root"))
            };
        };
        if let Some(previous) = self.entries[parent].last_child
            && let Content::Text(text) = &mut self.entries[previous].content
        {
            text.push_str(value);
            self.entries[previous].range.end = range.end;
        } else if !value.is_empty() {
            let _ = self.push(
                Content::Text(value.to_owned()),
                Some(parent),
                range,
                max_nodes,
            )?;
        }
        Ok(())
    }

    /// Return the sole root element; there is no synthetic document node.
    pub fn root_element(&self) -> Node<'_, 'input> {
        Node {
            document: self,
            index: self.root,
        }
    }

    #[cfg(test)]
    /// Iterate the root and all retained descendants in document order for tests.
    pub fn descendants(&self) -> impl Iterator<Item = Node<'_, 'input>> {
        (self.root..self.entries[self.root].subtree_end).map(|index| Node {
            document: self,
            index,
        })
    }
}

/// Copyable view of one arena entry, valid while its document remains borrowed.
/// `'document` bounds decoded values; `'input` bounds original source fragments.
#[derive(Clone, Copy)]
pub struct Node<'document, 'input> {
    document: &'document Document<'input>,
    index: usize,
}

impl<'document, 'input> Node<'document, 'input> {
    /// Whether this entry is an element rather than text or a placeholder.
    pub fn is_element(&self) -> bool {
        matches!(
            self.document.entries[self.index].content,
            Content::Element { .. }
        )
    }

    /// Whether this entry contains decoded text, including merged CDATA/references.
    pub fn is_text(&self) -> bool {
        matches!(self.document.entries[self.index].content, Content::Text(_))
    }

    /// Match an expanded element name exactly; non-elements never match.
    /// A string requires no namespace, while `(uri, local)` requires that URI.
    pub fn has_tag_name<'name>(&self, name: impl Into<ExpandedName<'name>>) -> bool {
        match &self.document.entries[self.index].content {
            Content::Element { name: actual, .. } => name.into().matches(actual),
            _ => false,
        }
    }

    /// Element name, or an empty unqualified sentinel for a non-element entry.
    pub fn tag_name(&self) -> &'document Name {
        match &self.document.entries[self.index].content {
            Content::Element { name, .. } => name,
            _ => &self.document.empty_name,
        }
    }

    /// Ordinary attributes in source order, excluding namespace declarations.
    /// Non-element entries yield an empty iterator.
    pub fn attributes(
        &self,
    ) -> impl Iterator<Item = &'document Attribute> + use<'document, 'input> {
        match &self.document.entries[self.index].content {
            Content::Element { attributes, .. } => attributes.as_slice(),
            _ => &[],
        }
        .iter()
    }

    /// Find a normalized attribute value by exact expanded name.
    /// A string selects an unqualified attribute, not any attribute with that local name.
    pub fn attribute<'name>(&self, name: impl Into<ExpandedName<'name>>) -> Option<&'document str> {
        let name = name.into();
        self.attributes()
            .find(|attribute| name.matches(&attribute.name))
            .map(Attribute::value)
    }

    /// Iterate direct children in source order, including non-element placeholders.
    pub fn children(&self) -> impl Iterator<Item = Self> + use<'document, 'input> {
        let document = self.document;
        std::iter::successors(document.entries[self.index].first_child, move |index| {
            document.entries[*index].next_sibling
        })
        .map(move |index| Self { document, index })
    }

    /// Find the first direct element child, skipping text and placeholders.
    pub fn first_element_child(&self) -> Option<Self> {
        self.children().find(Self::is_element)
    }

    /// Return this text node's value, or an element's first direct text child's value.
    ///
    /// Does not concatenate separated text children, descend into child elements,
    /// trim whitespace, or enforce scalar content. Callers needing a complete
    /// scalar must validate child structure and join the text children themselves.
    pub fn text(&self) -> Option<&'document str> {
        match &self.document.entries[self.index].content {
            Content::Text(text) => Some(text),
            Content::Element { .. } => self
                .children()
                .find(Self::is_text)
                .and_then(|node| node.text()),
            _ => None,
        }
    }

    #[cfg(test)]
    /// Iterate this entry and its retained descendants in preorder for tests.
    pub fn descendants(&self) -> impl Iterator<Item = Self> + use<'document, 'input> {
        let document = self.document;
        (self.index..document.entries[self.index].subtree_end)
            .map(move |index| Self { document, index })
    }

    /// Borrow the original source span, including an element's tags and descendants.
    /// Text spans retain original entity/CDATA syntax, not their normalized value.
    /// Inherited namespace declarations are not inserted into this fragment.
    pub fn source(&self) -> &'input str {
        &self.document.input[self.document.entries[self.index].range.clone()]
    }

    /// Collect effective explicit namespace declarations from this node and ancestors.
    ///
    /// Nearest declarations win per prefix; default-namespace resets are retained.
    /// Results follow the ancestor walk, not sorted prefix order. The implicit
    /// `xml` binding is not synthesized when no explicit declaration exists.
    pub fn namespaces(&self) -> Vec<&'document Namespace> {
        let mut namespaces: Vec<&Namespace> = Vec::new();
        let mut current = Some(self.index);
        while let Some(index) = current {
            let entry = &self.document.entries[index];
            if let Content::Element {
                namespaces: declarations,
                ..
            } = &entry.content
            {
                for namespace in declarations {
                    if !namespaces
                        .iter()
                        .any(|previous| previous.prefix == namespace.prefix)
                    {
                        namespaces.push(namespace);
                    }
                }
            }
            current = entry.parent;
        }
        namespaces
    }
}

/// Borrowed lookup key; absence of a namespace means unqualified, not a wildcard.
pub struct ExpandedName<'name>(Option<&'name str>, &'name str);

impl<'name> From<&'name str> for ExpandedName<'name> {
    fn from(name: &'name str) -> Self {
        Self(None, name)
    }
}

impl<'name> From<(&'name str, &'name str)> for ExpandedName<'name> {
    fn from((namespace, name): (&'name str, &'name str)) -> Self {
        Self(Some(namespace), name)
    }
}

impl ExpandedName<'_> {
    fn matches(&self, name: &Name) -> bool {
        self.0 == name.namespace() && self.1 == name.name()
    }
}

/// Validate a QName and resolve it using the reader's current namespace scope.
/// Attribute mode excludes the default namespace for unprefixed names.
fn resolve_name(
    reader: &NsReader<&[u8]>,
    qualified: quick_xml::name::QName<'_>,
    attribute: bool,
    budget: &mut ParseBudget,
) -> Result<Name, Error> {
    let raw =
        std::str::from_utf8(qualified.as_ref()).map_err(|_| Error::Invalid("name encoding"))?;
    validate_name(raw)?;
    let (namespace, local) = if attribute {
        reader.resolver().resolve_attribute(qualified)
    } else {
        reader.resolver().resolve_element(qualified)
    };
    budget.charge(local.as_ref().len())?;
    let namespace = match namespace {
        ResolveResult::Unbound => None,
        ResolveResult::Bound(namespace) => {
            let value = std::str::from_utf8(namespace.as_ref())
                .map_err(|_| Error::Invalid("namespace encoding"))?;
            Some(budget.namespace(value)?)
        }
        ResolveResult::Unknown(_) => return Err(Error::Invalid("undeclared namespace prefix")),
    };
    Ok(Name {
        namespace,
        local: std::str::from_utf8(local.as_ref())
            .map_err(|_| Error::Invalid("local name encoding"))?
            .to_owned(),
    })
}

/// Check XML 1.0 declaration fields and order without interpreting its encoding.
/// Placement at the document start is enforced by the parsing loop.
fn validate_declaration(declaration: &quick_xml::events::BytesDecl<'_>) -> Result<(), Error> {
    let content = std::str::from_utf8(declaration.as_ref())
        .map_err(|_| Error::Invalid("declaration encoding"))?;
    let element = quick_xml::events::BytesStart::from_content(content, 3);
    let mut attributes = element.attributes();
    let version = attributes
        .next()
        .ok_or(Error::Invalid("missing XML version"))?
        .map_err(quick_xml::Error::from)?;
    if version.key.as_ref() != b"version" || version.value.as_ref() != b"1.0" {
        return Err(Error::Invalid("unsupported XML version"));
    }
    let mut standalone_seen = false;
    for attribute in attributes {
        let attribute = attribute.map_err(quick_xml::Error::from)?;
        match attribute.key.as_ref() {
            b"encoding" if !standalone_seen => {
                let mut bytes = attribute.value.iter().copied();
                if !bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
                    || !bytes.all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
                    })
                {
                    return Err(Error::Invalid("invalid XML encoding name"));
                }
            }
            b"standalone" if matches!(attribute.value.as_ref(), b"yes" | b"no") => {
                standalone_seen = true;
            }
            _ => return Err(Error::Invalid("invalid XML declaration attribute")),
        }
    }
    Ok(())
}

/// Normalize literal attribute whitespace before expanding and checking references.
fn attribute_value(raw: &str) -> Result<String, Error> {
    let normalized = raw.replace("\r\n", " ").replace(['\r', '\n', '\t'], " ");
    let value = quick_xml::escape::unescape(&normalized).map_err(quick_xml::Error::from)?;
    if !value.chars().all(xml_character) {
        return Err(Error::Invalid("prohibited attribute character"));
    }
    Ok(value.into_owned())
}

/// XML 1.0 character repertoire, applied to both source and expanded references.
fn xml_character(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
}

/// XML 1.0 name-start character excluding the namespace separator colon.
fn name_start(character: char) -> bool {
    matches!(character, 'A'..='Z' | '_' | 'a'..='z' | '\u{c0}'..='\u{d6}' | '\u{d8}'..='\u{f6}' | '\u{f8}'..='\u{2ff}' | '\u{370}'..='\u{37d}' | '\u{37f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}

/// Validate one local name or a prefix/local pair, rejecting empty components.
/// Namespace binding and reserved-prefix checks are performed separately.
fn validate_name(name: &str) -> Result<(), Error> {
    let mut parts = name.split(':');
    for part in parts.by_ref().take(2) {
        let mut characters = part.chars();
        if !characters.next().is_some_and(name_start)
            || !characters.all(|character| name_start(character) || matches!(character, '-' | '.' | '0'..='9' | '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}'))
        {
            return Err(Error::Invalid("invalid qualified name"));
        }
    }
    if parts.next().is_some() {
        return Err(Error::Invalid("multiple namespace separators"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: many elements and attributes reuse a long inherited namespace URI.
    /// Guarantees: names share namespace storage and amplification fails within the byte budget.
    #[test]
    fn bounds_namespace_expansion() {
        let namespace = format!("urn:{}", "n".repeat(16_000));
        let input = format!("<root xmlns:p='{namespace}'><p:item p:key='value'/><p:item/></root>");
        let document = Document::parse(&input).unwrap();
        let children: Vec<_> = document.root_element().children().collect();
        let first = children[0].tag_name().namespace.as_ref().unwrap();
        let second = children[1].tag_name().namespace.as_ref().unwrap();
        let attribute = children[0].attributes().next().unwrap();
        assert!(Rc::ptr_eq(first, second));
        assert!(Rc::ptr_eq(
            first,
            attribute.name.namespace.as_ref().unwrap()
        ));
        for content in [
            "<p:item/>".repeat(2_000),
            (0..100)
                .map(|index| format!(" p:key{index}='value'"))
                .collect(),
        ] {
            let input = if content.starts_with(' ') {
                format!("<root xmlns:p='{namespace}'{content}/>")
            } else {
                format!("<root xmlns:p='{namespace}'>{content}</root>")
            };
            assert!(matches!(
                Document::parse(&input),
                Err(Error::Invalid("XML expanded byte budget exceeded"))
            ));
        }
    }

    /// Scenario: XML includes inherited namespaces, entities, CDATA, and normalized whitespace.
    /// Guarantees: expanded names, empty non-element names, scalar values, and source ranges are preserved.
    #[test]
    fn preserves_names_text_and_source() {
        let input = "<root xmlns='urn:root' xmlns:p='urn:p'><p:item p:key='a\r\n b&#13;' plain='v'>one&amp;<![CDATA[two\r\n]]>&#13;</p:item><plain xmlns=''/></root>";
        let document = Document::parse(input).unwrap();
        let root = document.root_element();
        let item = root.first_element_child().unwrap();
        assert!(item.has_tag_name(("urn:p", "item")));
        assert_eq!(item.attribute(("urn:p", "key")), Some("a  b\r"));
        assert_eq!(item.attribute("plain"), Some("v"));
        assert_eq!(item.text(), Some("one&two\n\r"));
        assert!(item.source().starts_with("<p:item"));
        assert!(item.source().ends_with("</p:item>"));
        assert_eq!(item.namespaces().len(), 2);
        assert!(
            root.children()
                .filter(Node::is_element)
                .last()
                .unwrap()
                .has_tag_name("plain")
        );
        let document = Document::parse("<root>text<!--comment--><?target value?></root>").unwrap();
        let children: Vec<_> = document.root_element().children().collect();
        assert_eq!(children.len(), 3);
        for child in children {
            assert!(!child.is_element());
            assert_eq!(child.tag_name().name(), "");
            assert_eq!(child.tag_name().namespace(), None);
            assert!(std::ptr::eq(child.tag_name(), &document.empty_name));
        }
    }

    /// Scenario: untrusted XML contains invalid syntax, entities, namespaces, or excessive nodes.
    /// Guarantees: malformed documents and DTDs fail closed before protocol interpretation.
    #[test]
    fn rejects_malformed_documents() {
        for input in [
            "",
            "<root>",
            "<root></other>",
            "<one/><two/>",
            "before<root/>",
            "<root/>after",
            "<!DOCTYPE root><root/>",
            "<root>&unknown;</root>",
            "<root>&#0;</root>",
            "<root>\u{1}</root>",
            "<p:root/>",
            "<root p:key='v'/>",
            "<root key='a' key='b'/>",
            "<root xmlns:a='urn:same' xmlns:b='urn:same' a:key='a' b:key='b'/>",
            "<root xmlns:xml='urn:wrong'/>",
            "<root xmlns:p=''/>",
            "<root xmlns='http://www.w3.org/XML/1998/namespace'/>",
            "<root key='<'/>",
            "<1root/>",
            "<a:b:c/>",
            "<root>]]></root>",
            "<root><!--bad--comment--></root>",
            "<root><?xml version='1.0'?></root>",
        ] {
            assert!(Document::parse(input).is_err(), "accepted {input}");
        }
        assert!(Document::parse_bounded("<root><child/></root>", 1).is_err());
    }

    /// Scenario: XML declarations contain optional fields, invalid ordering, or malformed values.
    /// Guarantees: only well-formed XML 1.0 declarations at the document start are accepted.
    #[test]
    fn validates_declarations() {
        for declaration in [
            "<?xml version='1.0'?>",
            "<?xml version='1.0' encoding='UTF-8' standalone='yes'?>",
            "<?xml version='1.0' standalone='no'?>",
        ] {
            assert!(Document::parse(&format!("{declaration}<root/>")).is_ok());
        }
        for declaration in [
            "<?xml?>",
            "<?xml version='1.1'?>",
            "<?xml version='1.0' extra='v'?>",
            "<?xml version='1.0' encoding=''?>",
            "<?xml version='1.0' encoding='1bad'?>",
            "<?xml version='1.0' standalone='true'?>",
            "<?xml version='1.0' standalone='no' encoding='UTF-8'?>",
            "<?xml version='1.0' encoding='UTF-8' encoding='UTF-8'?>",
            " <?xml version='1.0'?>",
            "<?xml version='1.0'?><?xml version='1.0'?>",
        ] {
            assert!(
                Document::parse(&format!("{declaration}<root/>")).is_err(),
                "accepted {declaration}"
            );
        }
    }

    /// Scenario: namespaces are escaped and shadowed, and documents have thousands of nested elements.
    /// Guarantees: names use decoded in-scope bindings and bounded parsing does not recurse.
    #[test]
    fn resolves_namespaces_and_bounds_deep_documents() {
        let document = Document::parse("<root xmlns:p='urn:a&amp;b'><p:item/><scope xmlns:p='urn:inner'><p:item xml:lang='en'/></scope></root>").unwrap();
        let root = document.root_element();
        assert!(
            root.first_element_child()
                .unwrap()
                .has_tag_name(("urn:a&b", "item"))
        );
        let item = root
            .children()
            .filter(Node::is_element)
            .last()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert!(item.has_tag_name(("urn:inner", "item")));
        assert_eq!(item.attribute((XML_NAMESPACE, "lang")), Some("en"));
        assert_eq!(item.namespaces()[0].uri(), "urn:inner");
        let input = format!("{}{}", "<root>".repeat(10_000), "</root>".repeat(10_000));
        assert!(Document::parse_bounded(&input, 100).is_err());
        assert_eq!(
            Document::parse_bounded(&input, 10_000)
                .unwrap()
                .descendants()
                .count(),
            10_000
        );
    }
}
