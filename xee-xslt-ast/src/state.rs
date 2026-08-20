use xee_datasource::{SpanInfo, SpanInfoKey, ValueRef, XmlDS};

use crate::{ast_core::Span, error::AttributeError, name::XmlName, names::Names};

/// Parser state affects the parsing output but does not change during parsing.
pub(crate) struct State<S: XmlDS> {
    pub(crate) source: S,
    pub(crate) span_info: SpanInfo<S>,
    pub(crate) names: Names<S>,
}

impl<S: XmlDS> State<S> {
    pub(crate) fn new(source: S, span_info: SpanInfo<S>, names: Names<S>) -> Self {
        Self {
            source,
            span_info,
            names,
        }
    }

    pub(crate) fn next(&self, node: S::Node) -> Option<S::Node> {
        self.source.next_sibling(node)
    }

    pub(crate) fn span(&self, node: S::Node) -> Option<Span> {
        use ValueRef::*;

        match self.source.value(node) {
            Element(_element) => self.span_info.get(SpanInfoKey::ElementStart(node)),
            Text(_text) => self.span_info.get(SpanInfoKey::Text(node)),
            Comment(_comment) => self.span_info.get(SpanInfoKey::Comment(node)),
            ProcessingInstruction(_pi) => self.span_info.get(SpanInfoKey::PiTarget(node)),
            Document => unreachable!(),
            // TODO: is it worthwhile to introduce span info keys for this?
            Attribute(_attribute) => unreachable!(),
            Namespace(_namespace) => unreachable!(),
        }
        .map(|span| span.into())
    }

    pub(crate) fn attribute_name_span(
        &self,
        node: S::Node,
        name: S::NameId,
    ) -> Result<Span, AttributeError> {
        let span = self
            .span_info
            .get(SpanInfoKey::AttributeName(node, name))
            .ok_or(AttributeError::Internal)?;
        Ok(span.into())
    }

    pub(crate) fn attribute_unexpected(
        &self,
        node: S::Node,
        name: S::NameId,
        _message: &str,
    ) -> AttributeError {
        let (local, namespace) = self.source.name_ns_str(name);
        let span = self.attribute_name_span(node, name);
        match span {
            Ok(span) => AttributeError::Unexpected {
                name: XmlName {
                    namespace: namespace.to_string(),
                    local: local.to_string(),
                },
                span,
            },
            Err(e) => e,
        }
    }
}
