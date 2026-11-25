// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opaque, bounded bookmark XML retained independently of its SOAP envelope.
//!
//! Cursor values are not interpreted, compared, or trusted as source identity.
//! The supported container is empty, scalar text, or one child element with
//! whitespace around it. Inner markup is retained verbatim. Inherited namespace
//! declarations are materialized on the root, so this is not a byte-identical
//! copy or a canonical representation of the incoming header.
//! The enclosing SOAP parser bounds input bytes, nodes, and namespace metadata;
//! serialization also bounds the detached UTF-8 output before each append.

use super::{BoundedXml, Error, WSMAN, reject_text, write_escaped_text};
use crate::common::xml::Node;
use quick_xml::{Reader, events::Event};
use std::io::{self, Write};

/// A validated, self-contained WS-Man Bookmark element, not committed progress.
#[derive(Debug, Eq, PartialEq)]
pub struct Bookmark(String);

impl Bookmark {
    /// Borrow the complete UTF-8 Bookmark element, including its namespace bindings.
    #[must_use]
    pub fn as_xml(&self) -> &str {
        &self.0
    }

    pub(super) fn from_node(node: Node<'_, '_>, limit: usize) -> Result<Self, Error> {
        if !node.has_tag_name((WSMAN, "Bookmark")) {
            return Err(Error::InvalidEnvelope("expected WS-Man Bookmark"));
        }
        let source = node.source();
        if source.len() > limit {
            return Err(Error::TooLarge);
        }
        if node.children().filter(Node::is_element).count() > 1 {
            return Err(Error::InvalidEnvelope("multiple bookmark roots"));
        }
        if node.first_element_child().is_some() {
            reject_text(node)?;
        }
        let mut reader = Reader::from_str(source);
        let first = reader
            .read_event()
            .map_err(|_| Error::InvalidEnvelope("invalid Bookmark root"))?;
        let empty = matches!(first, Event::Empty(_));
        let root = match first {
            Event::Start(root) | Event::Empty(root) => root,
            _ => return Err(Error::InvalidEnvelope("invalid Bookmark root")),
        };
        let suffix =
            &source[usize::try_from(reader.buffer_position()).map_err(|_| Error::TooLarge)?..];
        let mut output = BoundedXml {
            bytes: Vec::new(),
            limit,
        };
        let result = (|| -> io::Result<()> {
            output.write_all(b"<")?;
            output.write_all(root.as_ref().as_bytes())?;
            // Root declarations are already copied, and precede inherited bindings.
            // The shared parser has checked attribute uniqueness; do not rescan it here.
            let root_namespace_count = root.attributes().with_checks(false).try_fold(
                0usize,
                |count, attribute| -> io::Result<usize> {
                    Ok(count
                        + usize::from(
                            attribute
                                .map_err(io::Error::other)?
                                .key
                                .as_namespace_binding()
                                .is_some(),
                        ))
                },
            )?;
            for namespace in node.namespaces().skip(root_namespace_count) {
                if namespace
                    .name()
                    .is_some_and(|prefix| prefix.len().checked_add(6).is_none_or(|len| len > limit))
                {
                    return Err(io::Error::from(io::ErrorKind::FileTooLarge));
                }
                let name = namespace
                    .name()
                    .map_or_else(|| "xmlns".to_owned(), |prefix| format!("xmlns:{prefix}"));
                output.write_all(b" ")?;
                output.write_all(name.as_bytes())?;
                output.write_all(b"=\"")?;
                // Attribute whitespace must survive reparsing, including numeric references.
                for span in namespace.uri().split_inclusive(['\t', '\n']) {
                    if let Some(value) = span.strip_suffix('\t') {
                        write_escaped_text(&mut output, value)?;
                        output.write_all(b"&#9;")?;
                    } else if let Some(value) = span.strip_suffix('\n') {
                        write_escaped_text(&mut output, value)?;
                        output.write_all(b"&#10;")?;
                    } else {
                        write_escaped_text(&mut output, span)?;
                    }
                }
                output.write_all(b"\"")?;
            }
            output.write_all(if empty { b"/>" } else { b">" })?;
            output.write_all(suffix.as_bytes())
        })();
        result.map_err(|error| {
            if error.kind() == io::ErrorKind::FileTooLarge {
                Error::TooLarge
            } else {
                Error::ResponseWrite(error)
            }
        })?;
        String::from_utf8(output.bytes)
            .map(Self)
            .map_err(|_| Error::InvalidEncoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::xml::Document;

    fn extract(xml: &str, limit: usize) -> Result<Bookmark, Error> {
        let document = Document::parse(xml)?;
        Bookmark::from_node(
            document.root_element().first_element_child().unwrap(),
            limit,
        )
    }

    /// Scenario: a Bookmark inherits bindings, shadows prefixes, and contains escaped text and nested namespace resets.
    /// Guarantees: detaching preserves expanded names, attributes, opaque content, and empty/scalar bookmark forms.
    #[test]
    fn preserves_opaque_bookmark_and_namespace_context() {
        let inner = r#"<b:Cursor xml:lang="en" token="&quot;&amp;&#13;"><Child xmlns="">&#13;<![CDATA[<opaque>]]></Child><!--cursor--></b:Cursor>"#;
        let xml = format!(
            r#"<Envelope xmlns="urn:default" xmlns:w="{WSMAN}" xmlns:b="urn:wrong" xmlns:unused="urn:unused"><w:Bookmark xmlns:b="urn:cursor">{inner}</w:Bookmark></Envelope>"#
        );
        let bookmark = extract(&xml, 4096).unwrap();
        assert!(bookmark.as_xml().contains(inner));
        let detached = Document::parse(bookmark.as_xml()).unwrap();
        let root = detached.root_element();
        assert!(root.has_tag_name((WSMAN, "Bookmark")));
        let cursor = root.first_element_child().unwrap();
        assert!(cursor.has_tag_name(("urn:cursor", "Cursor")));
        assert_eq!(cursor.attribute("token"), Some("\"&\r"));
        assert_eq!(
            cursor.attribute(("http://www.w3.org/XML/1998/namespace", "lang")),
            Some("en")
        );
        let child = cursor.first_element_child().unwrap();
        assert!(child.has_tag_name("Child"));
        assert_eq!(child.text(), Some("\r<opaque>"));
        for content in ["", "token &amp; opaque&#13;"] {
            let xml = format!(r#"<E xmlns="{WSMAN}"><Bookmark>{content}</Bookmark></E>"#);
            let bookmark = extract(&xml, 4096).unwrap();
            let detached = Document::parse(bookmark.as_xml()).unwrap();
            assert!(detached.root_element().has_tag_name((WSMAN, "Bookmark")));
            assert_eq!(
                detached.root_element().text(),
                if content.is_empty() {
                    None
                } else {
                    Some("token & opaque\r")
                }
            );
        }
        let empty = extract(&format!(r#"<E xmlns:w="{WSMAN}"><w:Bookmark /></E>"#), 4096).unwrap();
        assert!(
            Document::parse(empty.as_xml())
                .unwrap()
                .root_element()
                .has_tag_name((WSMAN, "Bookmark"))
        );
    }

    /// Scenario: inherited namespace values contain escaped metacharacters and reference-encoded whitespace.
    /// Guarantees: streaming attribute escaping preserves namespace values on a second parse, without oversized temporary strings.
    #[test]
    fn preserves_escaped_namespace_values() {
        let uri = "urn:opaque:&amp;&quot;&apos;&lt;&gt;&#9;&#10;&#13;\u{e9}";
        let xml = format!(
            r#"<E xmlns:w="{WSMAN}" xmlns:c="{uri}"><w:Bookmark><c:Cursor/></w:Bookmark></E>"#
        );
        let original = Document::parse(&xml).unwrap();
        let node = original.root_element().first_element_child().unwrap();
        let expected = node
            .first_element_child()
            .unwrap()
            .tag_name()
            .namespace()
            .unwrap();
        let bookmark = extract(&xml, 4096).unwrap();
        let detached = Document::parse(bookmark.as_xml()).unwrap();
        assert_eq!(
            detached
                .root_element()
                .first_element_child()
                .unwrap()
                .tag_name()
                .namespace(),
            Some(expected)
        );
    }

    /// Scenario: input and namespace-expanded UTF-8 bookmarks reach their separate size limits.
    /// Guarantees: exact limits succeed, one byte less fails, and large inherited namespace expansion stays bounded.
    #[test]
    fn bounds_original_and_expanded_xml() {
        let standalone = format!(r#"<w:Bookmark xmlns:w="{WSMAN}"><Cursor/></w:Bookmark>"#);
        let xml = format!("<E>{standalone}</E>");
        assert_eq!(
            extract(&xml, standalone.len()).unwrap().as_xml(),
            standalone
        );
        assert!(matches!(
            extract(&xml, standalone.len() - 1),
            Err(Error::TooLarge)
        ));
        let inherited = format!(
            r#"<E xmlns:w="{WSMAN}" xmlns:c="urn:cursor"><w:Bookmark><c:C/></w:Bookmark></E>"#
        );
        let expanded = extract(&inherited, 4096).unwrap();
        assert_eq!(
            extract(&inherited, expanded.as_xml().len()).unwrap(),
            expanded
        );
        assert!(matches!(
            extract(&inherited, expanded.as_xml().len() - 1),
            Err(Error::TooLarge)
        ));
        let large = format!(
            r#"<E xmlns:w="{WSMAN}" xmlns:c="urn:{}"><w:Bookmark/></E>"#,
            "&amp;".repeat(100_000)
        );
        assert!(matches!(extract(&large, 256), Err(Error::TooLarge)));
        let large_prefix = format!(
            r#"<E xmlns:w="{WSMAN}" xmlns:{}="urn:cursor"><w:Bookmark/></E>"#,
            "p".repeat(100_000)
        );
        assert!(matches!(extract(&large_prefix, 256), Err(Error::TooLarge)));
        assert!(matches!(extract(&xml, 0), Err(Error::TooLarge)));
    }

    /// Scenario: a Bookmark has multiple payload roots, mixed content, or the wrong namespace.
    /// Guarantees: unsupported containers fail rather than becoming valid cursor candidates.
    #[test]
    fn rejects_invalid_bookmark_containers() {
        for content in ["<One/><Two/>", "text<One/>", "<One/>text"] {
            let xml = format!(r#"<E xmlns:w="{WSMAN}"><w:Bookmark>{content}</w:Bookmark></E>"#);
            assert!(matches!(
                extract(&xml, 4096),
                Err(Error::InvalidEnvelope(_))
            ));
        }
        assert!(matches!(
            extract("<E><Bookmark/></E>", 4096),
            Err(Error::InvalidEnvelope(_))
        ));
    }

    /// Scenario: a tiny bookmark inherits many distinct namespaces or declares its own bindings.
    /// Guarantees: oversized expansion fails at the output budget; local bindings are copied once and retain precedence.
    #[test]
    fn bounds_many_namespace_bindings() {
        let declarations: String = (0..100)
            .map(|index| format!(" xmlns:p{index}='urn:ancestor'"))
            .collect();
        let inherited = format!(r#"<E xmlns:w="{WSMAN}"{declarations}><w:Bookmark/></E>"#);
        assert!(matches!(extract(&inherited, 256), Err(Error::TooLarge)));
        let local =
            format!(r#"<w:Bookmark xmlns:w="{WSMAN}"{declarations}><p0:Cursor/></w:Bookmark>"#);
        let xml = format!("<E xmlns:p0='urn:shadowed'>{local}</E>");
        assert_eq!(extract(&xml, local.len()).unwrap().as_xml(), local);
    }
}
