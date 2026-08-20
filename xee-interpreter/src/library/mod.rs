/// XPath defines a standard function library, and this module implements
/// it.
mod accessor;
mod array;
mod boolean;
mod context;
mod datetime;
mod duration;
mod external;
mod fn_;
mod hidden_xslt;
mod hof;
mod id;
mod json;
mod map;
mod math;
mod node;
mod numeric;
mod parse;
mod qname;
mod sequence;
mod string;
mod uri;
mod xs;

use crate::function::{SourceSpecificFunctions, StaticFunctionDescription};
use xee_datasource::XmlDS;

pub(crate) fn static_function_descriptions<S: XmlDS + SourceSpecificFunctions>(
) -> Vec<StaticFunctionDescription<S>> {
    let mut descriptions = Vec::new();
    descriptions.extend(fn_::static_function_descriptions::<S>());
    descriptions.extend(string::static_function_descriptions::<S>());
    descriptions.extend(xs::static_function_descriptions::<S>());
    descriptions.extend(numeric::static_function_descriptions::<S>());
    descriptions.extend(math::static_function_descriptions::<S>());
    descriptions.extend(boolean::static_function_descriptions::<S>());
    descriptions.extend(accessor::static_function_descriptions::<S>());
    descriptions.extend(duration::static_function_descriptions::<S>());
    descriptions.extend(datetime::static_function_descriptions::<S>());
    descriptions.extend(sequence::static_function_descriptions::<S>());
    descriptions.extend(node::static_function_descriptions::<S>());
    descriptions.extend(qname::static_function_descriptions::<S>());
    descriptions.extend(context::static_function_descriptions::<S>());
    descriptions.extend(hof::static_function_descriptions::<S>());
    descriptions.extend(array::static_function_descriptions::<S>());
    descriptions.extend(map::static_function_descriptions::<S>());
    descriptions.extend(hidden_xslt::static_function_descriptions::<S>());
    descriptions.extend(uri::static_function_descriptions::<S>());
    descriptions.extend(external::static_function_descriptions::<S>());
    descriptions.extend(parse::static_function_descriptions::<S>());
    descriptions.extend(json::static_function_descriptions::<S>());
    descriptions.extend(id::static_function_descriptions::<S>());
    descriptions.extend(S::register_custom_functions());
    descriptions
}
