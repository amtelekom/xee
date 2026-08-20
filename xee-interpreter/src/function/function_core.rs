use std::rc::Rc;

use xee_datasource::XmlNode;

use crate::{context, stack};

use super::array::Array;
use super::map::Map;
use super::XmlDS;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct InlineFunctionId(pub(crate) usize);

impl InlineFunctionId {
    pub fn new(id: usize) -> Self {
        InlineFunctionId(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function<N: XmlNode> {
    Static(Rc<StaticFunctionData<N>>),
    Inline(Rc<InlineFunctionData<N>>),
    Map(Map<N>),
    Array(Array<N>),
}

#[derive(Debug, PartialEq)]
pub struct StaticFunctionData<N: XmlNode> {
    pub(crate) id: StaticFunctionId,
    pub(crate) closure_vars: Box<[stack::Value<N>]>,
}

impl<N: XmlNode> From<StaticFunctionData<N>> for Function<N> {
    fn from(data: StaticFunctionData<N>) -> Self {
        Self::Static(Rc::new(data))
    }
}

impl<N: XmlNode> StaticFunctionData<N> {
    pub(crate) fn new(id: StaticFunctionId, closure_vars: Vec<stack::Value<N>>) -> Self {
        StaticFunctionData {
            id,
            closure_vars: closure_vars.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InlineFunctionData<N: XmlNode> {
    pub(crate) id: InlineFunctionId,
    pub(crate) closure_vars: Box<[stack::Value<N>]>,
}

impl<N: XmlNode> From<InlineFunctionData<N>> for Function<N> {
    fn from(data: InlineFunctionData<N>) -> Self {
        Self::Inline(Rc::new(data))
    }
}

impl<N: XmlNode> InlineFunctionData<N> {
    pub(crate) fn new(id: InlineFunctionId, closure_vars: Vec<stack::Value<N>>) -> Self {
        InlineFunctionData {
            id,
            closure_vars: closure_vars.into(),
        }
    }
}

impl<N: XmlNode> Function<N> {
    pub(crate) fn closure_vars(&self) -> &[stack::Value<N>] {
        match self {
            Self::Static(data) => &data.closure_vars,
            Self::Inline(data) => &data.closure_vars,
            _ => unreachable!(),
        }
    }

    pub fn display_representation<S: XmlDS<Node = N>>(
        &self,
        source: &S,
        context: &context::DynamicContext<S>,
    ) -> String {
        match self {
            Self::Static(data) => {
                let function = context.static_function_by_id(data.id);
                function.display_representation()
            }
            Self::Inline(data) => {
                let function = context.inline_function_by_id(data.id);
                function.display_representation()
            }
            Self::Map(map) => map.display_representation(source, context),
            Self::Array(array) => array.display_representation(source, context),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "x86_64")]
    static_assertions::assert_eq_size!(super::Function<xot::Node>, [u8; 16]);
}
