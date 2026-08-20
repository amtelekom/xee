use crate::function::XmlDS;
use iri_string::types::{IriReferenceStr, IriString};
use xee_xpath_macros::xpath_fn;

use crate::{
    context::DynamicContext, error, function::StaticFunctionDescription, sequence::Sequence,
    wrap_xpath_fn,
};

#[xpath_fn("fn:doc($uri as xs:string?) as document-node()?")]
fn doc<S: XmlDS>(context: &DynamicContext<S>, uri: Option<&str>) -> error::Result<Option<S::Node>> {
    if let Some(uri) = uri {
        document_node(context, uri)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:doc-available($uri as xs:string?) as xs:boolean")]
fn doc_available<S: XmlDS>(context: &DynamicContext<S>, uri: Option<&str>) -> bool {
    if let Some(uri) = uri {
        document_node::<S>(context, uri).is_ok()
    } else {
        false
    }
}

fn document_node<S: XmlDS>(
    context: &DynamicContext<S>,
    uri: &str,
) -> error::Result<Option<S::Node>> {
    let iri_reference: &IriReferenceStr = uri.try_into().map_err(|_| error::Error::FODC0005)?;
    let uri = absolute_uri(context, iri_reference)?;

    // first check whether a document is there at all, if so, return it
    let documents = context.documents();
    let documents = documents.borrow();
    let document = documents.get_by_uri(&uri);

    if let Some(document) = document {
        Ok(Some(document.root()))
    } else {
        // The document doesn't exist, so return an error
        Err(error::Error::FODC0002)
    }
}

#[xpath_fn("fn:collection() as item()*")]
fn collection<S: XmlDS>(context: &DynamicContext<S>) -> error::Result<Sequence<S::Node>> {
    if let Some(collection) = context.default_collection() {
        Ok(collection.clone())
    } else {
        Err(error::Error::FODC0002)
    }
}

#[xpath_fn("fn:collection($uri as xs:string?) as item()*")]
fn collection_by_uri<S: XmlDS>(
    context: &DynamicContext<S>,
    uri: Option<&str>,
) -> error::Result<Sequence<S::Node>> {
    if let Some(uri) = uri {
        let iri_reference: &IriReferenceStr = uri.try_into().map_err(|_| error::Error::FODC0004)?;
        let uri = absolute_uri(context, iri_reference)?;
        if let Some(collection) = context.collection(&uri) {
            Ok(collection.clone())
        } else {
            Err(error::Error::FODC0002)
        }
    } else if let Some(collection) = context.default_collection() {
        Ok(collection.clone())
    } else {
        Err(error::Error::FODC0002)
    }
}

#[xpath_fn("fn:uri-collection() as xs:anyURI*")]
fn uri_collection<S: XmlDS>(context: &DynamicContext<S>) -> error::Result<Sequence<S::Node>> {
    if let Some(collection) = context.default_uri_collection() {
        Ok(collection.clone())
    } else {
        Err(error::Error::FODC0002)
    }
}

#[xpath_fn("fn:uri-collection($uri as xs:string?) as xs:anyURI*")]
fn uri_collection_by_uri<S: XmlDS>(
    context: &DynamicContext<S>,
    uri: Option<&str>,
) -> error::Result<Sequence<S::Node>> {
    if let Some(uri) = uri {
        let iri_reference: &IriReferenceStr = uri.try_into().map_err(|_| error::Error::FODC0004)?;
        let uri = absolute_uri(context, iri_reference)?;
        if let Some(collection) = context.uri_collection(&uri) {
            Ok(collection.clone())
        } else {
            Err(error::Error::FODC0002)
        }
    } else if let Some(collection) = context.default_uri_collection() {
        Ok(collection.clone())
    } else {
        Err(error::Error::FODC0002)
    }
}

fn absolute_uri<S: XmlDS>(
    context: &DynamicContext<S>,
    uri: &IriReferenceStr,
) -> error::Result<IriString> {
    let uri: IriString = match uri.to_iri() {
        Ok(iri) => iri.into(),
        Err(relative_iri) => {
            let base = context.static_context().static_base_uri();
            if let Some(base) = base {
                relative_iri.resolve_against(base).into()
            } else {
                return Err(error::Error::FODC0002);
            }
        }
    };
    Ok(uri)
}

#[xpath_fn("fn:environment-variable($name as xs:string) as xs:string?")]
fn environment_variable<S: XmlDS>(context: &DynamicContext<S>, name: &str) -> Option<String> {
    context.environment_variable(name).map(|s| s.to_string())
}

#[xpath_fn("fn:available-environment-variables() as xs:string*")]
fn available_environment_variables<S: XmlDS>(context: &DynamicContext<S>) -> Vec<String> {
    context
        .environment_variable_names()
        .map(|s| s.to_string())
        .collect()
}

// https://www.w3.org/TR/xpath-functions-31/#fns-on-docs
pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(doc, S),
        wrap_xpath_fn!(doc_available, S),
        wrap_xpath_fn!(collection, S),
        wrap_xpath_fn!(collection_by_uri, S),
        wrap_xpath_fn!(uri_collection, S),
        wrap_xpath_fn!(uri_collection_by_uri, S),
        wrap_xpath_fn!(environment_variable, S),
        wrap_xpath_fn!(available_environment_variables, S),
    ]
}
