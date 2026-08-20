// https://www.w3.org/TR/xpath-functions-31/#parsing-and-serializing

use xee_xpath_macros::xpath_fn;

use crate::function::XmlDS;
use crate::{context, error, function, interpreter::Interpreter, sequence, wrap_xpath_fn};

use super::StaticFunctionDescription;

#[xpath_fn("fn:parse-xml($arg as xs:string?) as document-node(element(*))?")]
fn parse_xml<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: Option<&str>,
) -> error::Result<Option<S::Node>> {
    if let Some(arg) = arg {
        let documents = context.documents();
        let handle = documents
            .borrow_mut()
            .add_string(interpreter.xot_mut(), None, arg)
            .map_err(|_| error::Error::FODC0006)?;
        let doc = documents
            .borrow()
            .get_node_by_handle(handle)
            .ok_or(error::Error::FODC0006)?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:parse-xml-fragment($arg as xs:string?) as document-node()?")]
fn parse_xml_fragment<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: Option<&str>,
) -> error::Result<Option<S::Node>> {
    if let Some(arg) = arg {
        let documents = context.documents();
        let handle = documents
            .borrow_mut()
            .add_fragment_string(interpreter.xot_mut(), arg)
            .map_err(|_| error::Error::FODC0006)?;
        let doc = documents
            .borrow()
            .get_node_by_handle(handle)
            .ok_or(error::Error::FODC0006)?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:serialize($arg as item()*) as xs:string")]
fn serialize1<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: &sequence::Sequence<S::Node>,
) -> error::Result<String> {
    let map = function::Map::new(vec![])?;
    let serialization_parameters = sequence::SerializationParameters::from_map(
        map,
        context.static_context(),
        interpreter.xot_mut(),
    )?;
    arg.serialize(serialization_parameters, interpreter.xot_mut())
}

#[xpath_fn("fn:serialize($arg as item()*, $params as item()?) as xs:string")]
fn serialize2<S: XmlDS>(
    context: &context::DynamicContext<S>,
    interpreter: &mut Interpreter<S>,
    arg: &sequence::Sequence<S::Node>,
    params: Option<sequence::Item<S::Node>>,
) -> error::Result<String> {
    let map = if let Some(params) = params {
        if let sequence::Item::Function(function::Function::Map(map)) = params {
            map.clone()
        } else {
            // TODO: handle element(output::serialization-parameters)
            return Err(error::Error::XPTY0004(
                "fn:serialize called with a $params argument that is not a map",
            ));
        }
    } else {
        function::Map::new(vec![])?
    };
    let serialization_parameters = sequence::SerializationParameters::from_map(
        map,
        context.static_context(),
        interpreter.xot_mut(),
    )?;
    arg.serialize(serialization_parameters, interpreter.xot_mut())
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(parse_xml, S),
        wrap_xpath_fn!(parse_xml_fragment, S),
        wrap_xpath_fn!(serialize1, S),
        wrap_xpath_fn!(serialize2, S),
    ]
}
