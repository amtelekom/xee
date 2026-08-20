use xee_interpreter::function::XmlDS;
use xee_xpath_compiler::context::Variables;

use crate::ast_core as ast;
use crate::error::ElementError as Error;
use crate::instruction::SequenceConstructorParser;
use crate::staticeval::static_evaluate;
use crate::{content::Content, context::Context, element::XsltParser, names::Names, state::State};

type Result<V> = std::result::Result<V, Error>;

pub fn parse_transform<S: XmlDS>(s: &str) -> Result<ast::Transform> {
    let mut source = S::new();
    let names = Names::new(&mut source);
    let (node, span_info) = source
        .parse_with_span_info(s)
        .map_err(|_e| Error::Unsupported(format!("Parse error: {}", _e)))?;
    let node = source.document_element(node).unwrap();
    let mut state = State::new(source, span_info, names);

    let mut source = S::new();
    static_evaluate(&mut state, node, Variables::<S>::new(), &mut source)
        .map_err(|_e| Error::Unsupported(format!("Static evaluate error: {:?}", _e)))?;
    let parser = XsltParser::new(&state);
    parser.parse_transform(node)
}

pub fn parse_sequence_constructor_item<S: XmlDS>(s: &str) -> Result<ast::SequenceConstructorItem> {
    let mut source = S::new();
    let names = Names::new(&mut source);
    let (node, span_info) = source.parse_with_span_info(s).unwrap();
    let state = State::new(source, span_info, names);
    let node = state.source.document_element(node).unwrap();

    if let Some(element) = state.source.element(node) {
        let context = Context::new(state.source.prefixes(node));
        let content = Content::new(node, &state, context);
        content.parse_element(
            element,
            ast::SequenceConstructorItem::parse_sequence_constructor_item,
        )
    } else {
        Err(Error::Internal)
    }
}
