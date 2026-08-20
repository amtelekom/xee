use xee_datasource::{Element, StringNewtype, ValueRef, XmlDS};
use xee_xpath_ast::XPathParserContext;

use crate::ast_core::Span;
use crate::ast_core::{self as ast};
use crate::attributes::Attributes;
use crate::combinator::{multi, one, NodeParser, OneParser};
use crate::content::Content;
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::{DeclarationParser, InstructionParser, SequenceConstructorParser};
use crate::state::State;
use crate::value_template::{ValueTemplateItem, ValueTemplateTokenizer};

pub(crate) struct XsltParser<'a, S: XmlDS> {
    state: &'a State<S>,
}

impl<'a, S: XmlDS> XsltParser<'a, S> {
    pub(crate) fn new(state: &'a State<S>) -> Self {
        Self { state }
    }

    pub(crate) fn parse_transform(&self, node: S::Node) -> Result<ast::Transform, ElementError> {
        let parser = instruction(self.state.names.xsl_transform)
            .or(instruction(self.state.names.xsl_stylesheet));
        parser.parse(Some(node), self.state, &Context::empty())
    }
}

impl<'a, S: XmlDS> Content<'a, S> {
    pub(crate) fn parse_element<V>(
        self,
        element: &'a S::Element,
        f: impl FnOnce(&Attributes<'a, S>) -> Result<V, ElementError>,
    ) -> Result<V, ElementError> {
        let attributes = self.attributes(element).with_standard()?;
        f(&attributes)
    }

    pub(crate) fn span(&self) -> Result<Span, ElementError> {
        self.state.span(self.node).ok_or(ElementError::Internal)
    }

    pub(crate) fn parser_context(&self) -> XPathParserContext {
        self.context.parser_context(self.state)
    }

    pub(crate) fn sequence_constructor(&self) -> Result<ast::SequenceConstructor, ElementError> {
        self.lazy_init
            .sequence_constructor
            .get_or_init(|| children(sequence_constructor()))(self)
    }

    pub(crate) fn declarations(&self) -> Result<ast::Declarations, ElementError> {
        self.lazy_init
            .declarations_content
            .get_or_init(|| children(declarations()))(self)
    }
}

pub(crate) fn sequence_constructor<S: XmlDS>() -> impl NodeParser<ast::SequenceConstructor, S> {
    multi(|content: Content<'_, S>| {
        let node = content.node;
        let state = &content.state;
        let context = &content.context;
        match state.source.value(node) {
            ValueRef::Text(text) => {
                let span = state.span(node).ok_or(ElementError::Internal)?;
                let parser_context = content.parser_context();
                if context.expand_text {
                    text_value_template(text.get(), span, &parser_context)
                } else {
                    Ok(vec![ast::SequenceConstructorItem::Content(
                        ast::Content::Text(text.get().to_string()),
                    )])
                }
            }
            ValueRef::Element(element) => content.parse_element(element, |attributes| {
                Ok(vec![
                    ast::SequenceConstructorItem::parse_sequence_constructor_item(attributes)?,
                ])
            }),
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        }
    })
    .flatten()
}

fn declarations<S: XmlDS>() -> impl NodeParser<ast::Declarations, S> {
    one(|content: Content<'_, S>| {
        match content.state.source.value(content.node) {
            ValueRef::Element(element) => content.parse_element(element, |attributes| {
                ast::Declaration::parse_declaration(attributes)
            }),
            _ => Err(ElementError::Unexpected {
                // TODO: get span right
                span: Span::new(0, 0),
            }),
        }
    })
    .many()
}

fn text_value_template(
    s: &str,
    span: Span,
    parser_context: &XPathParserContext,
) -> Result<Vec<ast::SequenceConstructorItem>, ElementError> {
    let mut items = Vec::new();
    for token in ValueTemplateTokenizer::new(s, span, parser_context) {
        let token = token?;
        let content = match token {
            ValueTemplateItem::String { text, span: _ } => ast::Content::Text(text.to_string()),
            ValueTemplateItem::Curly { c } => ast::Content::Text(c.to_string()),
            ValueTemplateItem::Value { xpath, span } => {
                ast::Content::Value(Box::new(ast::Expression { xpath, span }))
            }
        };
        let item = ast::SequenceConstructorItem::Content(content);
        items.push(item);
    }
    Ok(items)
}

#[allow(type_alias_bounds)]
type ContentParse<V, S: XmlDS> =
    Box<dyn Fn(&Content<S>) -> Result<V, ElementError> + std::marker::Sync + std::marker::Send>;

pub(crate) fn children<V, P, S: XmlDS>(parser: P) -> ContentParse<V, S>
where
    P: NodeParser<V, S> + std::marker::Sync + std::marker::Send + 'static,
    V: std::marker::Sync + std::marker::Send + 'static,
{
    Box::new(move |content| {
        let (item, next) = parser.parse_next(
            content.state.source.first_child(content.node),
            content.state,
            &content.context,
        )?;
        // handle end of content check here
        if let Some(next) = next {
            Err(ElementError::Unexpected {
                span: content.state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    })
}

pub(crate) fn by_element<V, S: XmlDS>(
    f: impl Fn(&Attributes<S>) -> Result<V, ElementError>,
) -> impl Fn(Content<S>) -> Result<V, ElementError> {
    move |content| {
        let element =
            content
                .state
                .source
                .element(content.node)
                .ok_or(ElementError::Unexpected {
                    span: content
                        .state
                        .span(content.node)
                        .ok_or(ElementError::Internal)?,
                })?;
        content.parse_element(element, &f)
    }
}

pub(crate) fn by_element_name<V, S: XmlDS>(
    name: S::NameId,
    f: impl Fn(&Attributes<S>) -> Result<V, ElementError>,
) -> impl Fn(Content<S>) -> Result<V, ElementError> {
    by_element(move |attributes: &Attributes<'_, S>| {
        if attributes.element.name() == name {
            f(attributes)
        } else {
            Err(ElementError::Unexpected {
                span: attributes.content.span()?,
            })
        }
    })
}

pub(crate) fn by_instruction<V: InstructionParser<S>, S: XmlDS>(
    name: S::NameId,
) -> impl Fn(Content<S>) -> Result<V, ElementError> {
    by_element_name(name, move |attributes| V::parse_and_validate(attributes))
}

pub(crate) fn instruction<V: InstructionParser<S>, S: XmlDS>(
    name: S::NameId,
) -> OneParser<V, impl Fn(Content<S>) -> Result<V, ElementError>, S> {
    one(by_instruction(name))
}
