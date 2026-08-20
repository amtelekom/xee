/// XPath can be extended with both static functions as well as user defined
/// functions.
mod array;
mod function_core;
mod inline_function;
mod map;
mod signature;
mod static_function;

pub use array::Array;
pub use function_core::Function;
pub use function_core::{
    InlineFunctionData, InlineFunctionId, StaticFunctionData, StaticFunctionId,
};
pub use inline_function::{CastType, InlineFunction, Name};
pub use map::Map;
pub use signature::Signature;

pub use static_function::FunctionRule;
pub use static_function::SourceSpecificFunctions;
pub use static_function::StaticFunctionType;
pub use static_function::{FunctionKind, StaticFunctionDescription};
pub use static_function::{StaticFunction, StaticFunctions};

/// inside of xee-interpreter, we nearly always want to refer to XmlDS + SourceSpecificFunctions,
/// which equals just requiring SourceSpecificFunctions (because it depends on XmlDS),
/// but semantically we're much more interested in the XmlDS part,
/// so this shorthand helps keep things tidy.
pub use static_function::SourceSpecificFunctions as XmlDS;
