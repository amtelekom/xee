use ibig::IBig;

use xee_datasource::XmlNode;
use xee_name::Name;
use xee_xpath_macros::xpath_fn;

use crate::error;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:my_function($a as xs:integer, $b as xs:integer) as xs:integer")]
fn my_function(a: IBig, b: IBig) -> IBig {
    a + b
}

// FIXME: this is not the right signature for fn:error, as it always returns a
// none.
//
// According to the spec: The type "none" is a special type defined in [XQuery
// 1.0 and XPath 2.0 Formal Semantics] and is not available to the user. It
// indicates that the function never returns and ensures that it has the
// correct static type.
//
// So this support should be added.
#[xpath_fn("fn:error() as item()*")]
fn error_zero_args<N: XmlNode>() -> error::Result<sequence::Sequence<N>> {
    Err(error::Error::FOER0000)
}

#[xpath_fn("fn:error($code as xs:QName?) as item()*")]
fn error_with_code<N: XmlNode>(code: Option<Name>) -> error::Result<sequence::Sequence<N>> {
    if let Some(code) = code {
        Err(error::Error::Application(Box::new(
            error::ApplicationError::new(code, "".to_string()),
        )))
    } else {
        Err(error::Error::FOER0000)
    }
}

#[xpath_fn("fn:error($code as xs:QName?, $description as xs:string) as item()*")]
fn error_with_code_and_description<N: XmlNode>(
    code: Option<Name>,
    description: &str,
) -> error::Result<sequence::Sequence<N>> {
    error_helper(code, description)
}

#[xpath_fn(
    "fn:error($code as xs:QName?, $description as xs:string, $error_object as item()*) as item()*"
)]
fn error_with_code_and_description_and_sequence<N: XmlNode>(
    code: Option<Name>,
    description: &str,
    _error_object: &sequence::Sequence<N>,
) -> error::Result<sequence::Sequence<N>> {
    // FIXME: we are not doing anything with _error_object
    error_helper(code, description)
}

fn error_helper<N: XmlNode>(
    code: Option<Name>,
    description: &str,
) -> error::Result<sequence::Sequence<N>> {
    if let Some(code) = code {
        Err(error::Error::Application(Box::new(
            error::ApplicationError::new(code, description.to_string()),
        )))
    } else {
        let unknown_error_qname = Name::new(
            "FOER0000".to_string(),
            "http://www.w3.org/2005/xqt-errors".to_string(),
            "".to_string(),
        );
        Err(error::Error::Application(Box::new(
            error::ApplicationError::new(unknown_error_qname, description.to_string()),
        )))
    }
}

#[xpath_fn("fn:trace($value as item()*) as item()*")]
fn trace<N: XmlNode>(value: &sequence::Sequence<N>) -> sequence::Sequence<N> {
    // TODO: direct values to the "trace data set".
    value.clone()
}

#[xpath_fn("fn:trace($value as item()*,$label as xs:string) as item()*")]
fn trace_with_label<N: XmlNode>(
    value: &sequence::Sequence<N>,
    _label: &str,
) -> sequence::Sequence<N> {
    // TODO: direct values + label to the "trace data set".
    value.clone()
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(my_function, S),
        wrap_xpath_fn!(error_zero_args, S),
        wrap_xpath_fn!(error_with_code, S),
        wrap_xpath_fn!(error_with_code_and_description, S),
        wrap_xpath_fn!(error_with_code_and_description_and_sequence, S),
        wrap_xpath_fn!(trace, S),
        wrap_xpath_fn!(trace_with_label, S),
    ]
}
