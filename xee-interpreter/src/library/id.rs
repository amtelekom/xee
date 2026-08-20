use ahash::{HashSet, HashSetExt};
use xee_xpath_macros::xpath_fn;

use crate::context::DynamicContext;
use crate::error::Error;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::interpreter::Interpreter;
use crate::{wrap_xpath_fn, xml};

#[xpath_fn(
    "fn:id($arg as xs:string*, $node as node()) as element()*",
    context_last
)]
fn id<S: XmlDS>(
    context: &DynamicContext<S>,
    interpreter: &Interpreter<S>,
    arg: impl Iterator<Item = Result<String, Error>>,
    node: S::Node,
) -> Result<Vec<S::Node>, Error> {
    ids_helper(
        arg,
        node,
        interpreter.source(),
        context
            .documents()
            .borrow()
            .document_order_access(interpreter.source()),
    )
}

#[xpath_fn(
    "fn:element-with-id($arg as xs:string*, $node as node()) as element()*",
    context_last
)]
fn element_with_id<S: XmlDS>(
    context: &DynamicContext<S>,
    interpreter: &Interpreter<S>,
    arg: impl Iterator<Item = Result<String, Error>>,
    node: S::Node,
) -> Result<Vec<S::Node>, Error> {
    // we only support xml:id so in the absence of schema information that
    // identifies an ID element, the behavior is the same as for fn:id
    ids_helper(
        arg,
        node,
        interpreter.source(),
        context
            .documents()
            .borrow()
            .document_order_access(interpreter.source()),
    )
}

fn ids_helper<S: XmlDS>(
    arg: impl Iterator<Item = Result<String, Error>>,
    node: S::Node,
    source: &S,
    annotations: xml::DocumentOrderAccess<S>,
) -> Result<Vec<S::Node>, Error> {
    let document_node = source.root(node);
    let mut result: Vec<S::Node> = Vec::new();
    let mut seen = HashSet::new();
    for idrefs in arg {
        let idrefs = idrefs?;
        // split idrefs into individual ids
        for idref in idrefs.split_whitespace() {
            if seen.contains(idref) {
                continue;
            }
            seen.insert(idref.to_string());
            // find the element with the given id
            // if found, return it
            // if not found, return an empty sequence
            if let Some(node) = source.xml_id_node(document_node, idref) {
                result.push(node);
            }
        }
    }
    result.sort_by_key(|n| annotations.get(*n));
    Ok(result)
}

#[xpath_fn("fn:generate-id($arg as node()?) as xs:string", context_first)]
fn generate_id<S: XmlDS>(
    context: &DynamicContext<S>,
    interpreter: &Interpreter<S>,
    arg: Option<S::Node>,
) -> String {
    if let Some(arg) = arg {
        let documents = context.documents();
        let documents = documents.borrow();
        let annotations = documents.document_order_access(interpreter.source());
        let annotation = annotations.get(arg);
        annotation.generate_id()
    } else {
        "".to_string()
    }
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(id, S),
        wrap_xpath_fn!(element_with_id, S),
        wrap_xpath_fn!(generate_id, S),
    ]
}
