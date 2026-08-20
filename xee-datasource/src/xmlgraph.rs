// The following is based on the implementation from XOT and slightly modified to work with the generic traits:
// Copyright (c) 2022 Martijn Faassen & Paligo

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use ahash::HashMap;

use crate::{Span, XmlDS, XmlId};

use super::XmlNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEdge<N: XmlNode> {
    /// The start edge of a node. In case of an element
    /// this is the start tag. In case of document this is
    /// the start of the document.
    Start(N),
    /// The end edge of a node. In case of an element
    /// this is the end tag. In case of document this is the end
    /// of the document. For any other values, the
    /// end edge occurs immediately after the start
    /// edge.
    End(N),
}

/// A key to use to look up span information using
/// [`SpanInfo::get`](`crate::SpanInfo::get`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanInfoKey<N: XmlNode, I: XmlId> {
    /// The name part of an attribute.
    /// In `foo:name="value"`, the `foo:name` part
    AttributeName(N, I),
    /// The value part of an attribute.
    /// In `foo:name="value"`, the `value` part
    AttributeValue(N, I),
    /// The name part of a start element tag.
    /// In `<foo:name ..>`, the `foo:name` part
    ElementStart(N),
    /// The closing part of the end element tag (or a self-closing element).
    /// In `</foo:name>`, the `</foo:name>` part, or if it is an empty element
    /// `<foo:name/>`, the `/>` part
    ElementEnd(N),
    /// Text node.
    /// In `<foo>text</foo>`, the `text` part
    Text(N),
    /// Comment node.
    /// In `<!--comment-->`, the `comment` part
    Comment(N),
    /// The target part of a processing instruction.
    /// In `<?target content?>`, the `target` part
    PiTarget(N),
    /// The content part of a processing instruction (if defined).
    /// In `<?target content?>`, the `content` part
    PiContent(N),
}

/// Span information for a parsed XML document.
///
/// This span information is valid immediately after the parse. It becomes
/// invalid as soon as you mutate the parsed document.
///
/// You obtain this by using
/// [`Xot::parse_with_span_info`](`Xot::parse_with_span_info`).
///
/// You use a [`SpanInfoKey`](crate::SpanInfoKey) to look up the span
/// information.
///
/// The span of a CDATA section is only its text content:
///
/// ```text
/// <p><![CDATA[content]]></p>
///             ^^^^^^^
/// ```
///
/// There is an exception to this. During parsing, adjacent CDATA sections and
/// text nodes are consolidated into a single text node. This text node has the
/// span starting with the first adjacent CDATA section or text node and ending
/// with the last adjacent CDATA section or text node.
///
/// Example:
///
/// ```text
/// <p>text<![CDATA[content]]>text</p>
///    ^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// This can lead to the slightly odd situation where only part of the CDATA
/// marker is included in the span:
///
/// ```text
/// <p>text<![CDATA[content]]>foo</p>
///                 ^^^^^^^^^^^^^
///```
///
/// In every case all text content in the adjacent CDATA and text is included
/// in the span.
pub struct SpanInfo<S: XmlDS> {
    pub(crate) map: HashMap<SpanInfoKey<S::Node, S::NameId>, super::Span>,
}

impl<S: XmlDS> SpanInfo<S> {
    // fn new() -> Self {
    //     SpanInfo {
    //         map: HashMap::new(),
    //     }
    // }

    /// Get span info by [`SpanInfoKey`](crate::SpanInfoKey)
    pub fn get(&self, key: SpanInfoKey<S::Node, S::NameId>) -> Option<&Span> {
        self.map.get(&key)
    }

    // fn add(&mut self, key: SpanInfoKey<S::Node, S::NameId>, span: Span) {
    //     self.map.insert(key, span);
    // }

    // fn extend_text_span(&mut self, node: S::Node, span: Span) {
    //     // if we already have span for this (text) node it, we need to store the span with that
    //     // start and the given ending
    //     let key = SpanInfoKey::Text(node);
    //     if let Some(existing_span) = self.map.get(&key) {
    //         let start = existing_span.start;
    //         let end = span.end;
    //         self.map.insert(key, Span::new(start, end));
    //     } else {
    //         self.map.insert(key, span);
    //     }
    // }

    // fn add_attribute_spans(&mut self, node_id: S::NodeId, attribute_spans: AttributeSpans) {
    //     for (attribute_name, name_span, value_span) in attribute_spans {
    //         self.add(
    //             SpanInfoKey::AttributeName(node_id.into(), attribute_name),
    //             name_span,
    //         );
    //         self.add(
    //             SpanInfoKey::AttributeValue(node_id.into(), attribute_name),
    //             value_span,
    //         );
    //     }
    // }
}
