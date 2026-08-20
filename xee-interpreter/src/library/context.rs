//! https://www.w3.org/TR/2017/REC-xpath-functions-31-20170321/#context

use xee_name::{Name, Namespaces, FN_NAMESPACE};
use xee_xpath_ast::ast;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::atomic::NaiveDateWithOffset;
use crate::atomic::NaiveTimeWithOffset;
use crate::context::DynamicContext;
use crate::error;
use crate::function::FunctionKind;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;

use super::datetime::offset_to_duration;

fn bound_position<S: XmlDS>(
    _context: &DynamicContext<S>,
    _interpreter: &mut interpreter::Interpreter<S>,
    arguments: &[sequence::Sequence<S::Node>],
) -> error::Result<sequence::Sequence<S::Node>> {
    // position should be the context value
    Ok(arguments[0].clone())
}

fn bound_last<S: XmlDS>(
    _context: &DynamicContext<S>,
    _interpreter: &mut interpreter::Interpreter<S>,
    arguments: &[sequence::Sequence<S::Node>],
) -> error::Result<sequence::Sequence<S::Node>> {
    // size should be the context value
    Ok(arguments[0].clone())
}

#[xpath_fn("fn:current-dateTime() as xs:dateTimeStamp")]
fn current_date_time<S: XmlDS>(
    context: &DynamicContext<S>,
) -> chrono::DateTime<chrono::offset::FixedOffset> {
    context.current_datetime()
}

#[xpath_fn("fn:current-date() as xs:date")]
fn current_date<S: XmlDS>(context: &DynamicContext<S>) -> NaiveDateWithOffset {
    NaiveDateWithOffset {
        date: context.current_datetime().naive_local().date(),
        offset: Some(context.implicit_timezone()),
    }
}

#[xpath_fn("fn:current-time() as xs:time")]
fn current_time<S: XmlDS>(context: &DynamicContext<S>) -> NaiveTimeWithOffset {
    NaiveTimeWithOffset {
        time: context.current_datetime().time(),
        offset: Some(context.implicit_timezone()),
    }
}

#[xpath_fn("fn:implicit-timezone() as xs:dayTimeDuration")]
fn implicit_timezone<S: XmlDS>(context: &DynamicContext<S>) -> chrono::Duration {
    offset_to_duration(context.implicit_timezone())
}

#[xpath_fn("fn:default-collation() as xs:string")]
fn default_collation<S: XmlDS>(context: &DynamicContext<S>) -> String {
    context.static_context().default_collation_uri().to_string()
}

#[xpath_fn("fn:static-base-uri() as xs:anyURI?")]
fn static_base_uri<S: XmlDS>(context: &DynamicContext<S>) -> Option<atomic::Atomic> {
    context
        .static_context()
        .static_base_uri()
        .map(|uri| atomic::Atomic::String(atomic::StringType::AnyURI, uri.to_string().into()))
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        StaticFunctionDescription::<S> {
            name: Name::new(
                "position".to_string(),
                FN_NAMESPACE.to_string(),
                String::new(),
            ),
            signature: ast::Signature::parse("fn:position() as xs:integer", &Namespaces::default())
                .unwrap()
                .into(),
            function_kind: Some(FunctionKind::Position),
            func: bound_position,
        },
        StaticFunctionDescription::<S> {
            name: Name::new("last".to_string(), FN_NAMESPACE.to_string(), String::new()),
            signature: ast::Signature::parse("fn:last() as xs:integer", &Namespaces::default())
                .unwrap()
                .into(),
            function_kind: Some(FunctionKind::Size),
            func: bound_last,
        },
        wrap_xpath_fn!(current_date_time, S),
        wrap_xpath_fn!(current_date, S),
        wrap_xpath_fn!(current_time, S),
        wrap_xpath_fn!(implicit_timezone, S),
        wrap_xpath_fn!(default_collation, S),
        wrap_xpath_fn!(static_base_uri, S),
    ]
}
