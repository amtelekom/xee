//! https://www.w3.org/TR/xpath-functions-31/#accessors

use xee_datasource::xmlname::RefName;
use xee_xpath_ast::ast;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context;
use crate::error;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::interpreter::Interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::xml::BaseUriResolver;

#[xpath_fn("fn:node-name($arg as node()?) as xs:QName?", context_first)]
fn node_name<S: XmlDS>(
    interpreter: &Interpreter<S>,
    arg: Option<S::Node>,
) -> error::Result<Option<ast::Name>> {
    Ok(if let Some(node) = arg {
        interpreter
            .source()
            .node_name_ref(node)?
            .map(|n| n.to_owned())
    } else {
        None
    })
}

#[xpath_fn("fn:string($arg as item()?) as xs:string", context_first)]
fn string<S: XmlDS>(
    interpreter: &Interpreter<S>,
    arg: Option<sequence::Item<S::Node>>,
) -> error::Result<String> {
    if let Some(arg) = arg {
        arg.string_value(interpreter.source())
    } else {
        Ok("".to_string())
    }
}

#[xpath_fn("fn:data($arg as item()*) as xs:anyAtomicType*", context_first)]
fn data<S: XmlDS>(
    interpreter: &Interpreter<S>,
    arg: &sequence::Sequence<S::Node>,
) -> error::Result<Vec<sequence::Item<S::Node>>> {
    let data = arg
        .atomized(interpreter.source())
        .map(|atom| atom.map(|a| a.into()))
        .collect::<error::Result<Vec<sequence::Item<S::Node>>>>()?;
    Ok(data)
}

#[xpath_fn("fn:base-uri($arg as node()?) as xs:anyURI?", context_first)]
fn base_uri<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: Option<S::Node>,
) -> error::Result<Option<atomic::Atomic>> {
    Ok(if let Some(node) = arg {
        // root node of the document
        let root = interpreter.source().root(node);

        let base_uri = if matches!(
            interpreter.source().value(root),
            xee_datasource::ValueRef::Document
        ) {
            // the base uri of the document is the one we can find registered, if available
            let documents = context.documents();
            let documents = documents.borrow();
            documents.get_uri_by_document_node(root)
        } else {
            None
        };

        // if we don't have a registered URI, use the static base uri
        let base_uri = base_uri.or_else(|| {
            context
                .static_context()
                .static_base_uri()
                .map(|u| u.to_owned().into())
        });
        let resolver = BaseUriResolver::new(base_uri.as_deref(), interpreter.state.source_mut());
        let base_iri = resolver.base_uri(node)?;
        base_iri.map(|i| i.into())
    } else {
        None
    })
}

#[xpath_fn("fn:document-uri($arg as node()?) as xs:anyURI?", context_first)]
fn document_uri<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: Option<S::Node>,
) -> error::Result<Option<atomic::Atomic>> {
    Ok(if let Some(node) = arg {
        // root node of the document
        let root = interpreter.source().root(node);

        if matches!(
            interpreter.source().value(root),
            xee_datasource::ValueRef::Document
        ) {
            // the base uri of the document is the one we can find registered, if available
            let documents = context.documents();
            let documents = documents.borrow();
            let document_uri = documents.get_uri_by_document_node(root);
            document_uri.map(|document_uri| document_uri.into())
        } else {
            None
        }
    } else {
        None
    })
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(node_name, S),
        wrap_xpath_fn!(string, S),
        wrap_xpath_fn!(data, S),
        wrap_xpath_fn!(base_uri, S),
        wrap_xpath_fn!(document_uri, S),
    ]
}
