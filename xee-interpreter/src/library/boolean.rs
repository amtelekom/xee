//! https://www.w3.org/TR/xpath-functions-31/#boolean-functions

use xee_datasource::XmlNode;
use xee_xpath_macros::xpath_fn;

use crate::error;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:true() as xs:boolean")]
fn true_() -> bool {
    true
}

#[xpath_fn("fn:false() as xs:boolean")]
fn false_() -> bool {
    false
}

#[xpath_fn("fn:not($arg as item()*) as xs:boolean")]
fn not<N: XmlNode>(arg: &sequence::Sequence<N>) -> error::Result<bool> {
    arg.effective_boolean_value().map(|b| !b)
}

#[xpath_fn("fn:boolean($arg as item()*) as xs:boolean")]
fn boolean<N: XmlNode>(arg: &sequence::Sequence<N>) -> error::Result<bool> {
    arg.effective_boolean_value()
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(true_, S),
        wrap_xpath_fn!(false_, S),
        wrap_xpath_fn!(not, S),
        wrap_xpath_fn!(boolean, S),
    ]
}
