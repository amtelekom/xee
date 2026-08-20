use xee_interpreter::function::XmlDS;
use xee_name::{Namespaces, FN_NAMESPACE};

use xee_interpreter::context::StaticContext;
use xee_interpreter::error;
use xee_interpreter::interpreter::Program;
use xee_interpreter::sequence;

use crate::ast_ir::parse;

pub fn evaluate_program<S: XmlDS>(
    xot: &mut S,
    program: &Program<S>,
    root: S::Node,
) -> error::SpannedResult<sequence::Sequence<S::Node>> {
    let mut documents = xee_interpreter::xml::Documents::new();
    let handle = documents.add_root(None, root).unwrap();
    let root = documents.get_node_by_handle(handle).unwrap();
    let mut dynamic_context_builder = program.dynamic_context_builder();
    dynamic_context_builder.context_node(root);
    dynamic_context_builder.documents(documents);
    let context = dynamic_context_builder.build();
    let runnable = program.runnable(&context);
    runnable.many(xot)
}

pub fn evaluate<S: XmlDS>(
    xot: &mut S,
    xml: &str,
    xslt: &str,
) -> error::SpannedResult<sequence::Sequence<S::Node>> {
    let namespaces = Namespaces::new(
        Namespaces::default_namespaces(),
        "".to_string(),
        FN_NAMESPACE.to_string(),
    );
    let static_context = StaticContext::from_namespaces(namespaces);
    let root = xot.parse(xml).unwrap();
    let program = parse(static_context, xslt).unwrap();
    evaluate_program(xot, &program, root)
}
