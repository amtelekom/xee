use xee_datasource::XmlDS;
use xee_schema_type::Xs;
use xee_xpath_type::ast::SequenceType;

use crate::sequence;
use crate::span::SourceSpan;
use crate::xml;

use super::signature::Signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastType {
    pub xs: Xs,
    pub empty_sequence_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub(crate) String);

impl Name {
    pub fn new(name: String) -> Self {
        Name(name)
    }
}

#[derive(Debug, Clone)]
pub struct InlineFunction<S: XmlDS> {
    pub name: String,
    pub signature: Signature,
    // things referenced by instructions (by index)
    pub constants: Vec<sequence::Sequence<S::Node>>,
    pub steps: Vec<xml::Step>,
    pub cast_types: Vec<CastType>,
    pub sequence_types: Vec<SequenceType>,
    pub closure_names: Vec<Name>,
    // the compiled code, and the spans of each instruction
    pub chunk: Vec<u8>,
    pub spans: Vec<SourceSpan>,
}

impl<S: XmlDS> InlineFunction<S> {
    pub(crate) fn signature(&self) -> &Signature {
        &self.signature
    }

    pub(crate) fn arity(&self) -> usize {
        self.signature.parameter_types().len()
    }

    pub fn display_representation(&self) -> String {
        let signature = self.signature.display_representation();
        format!("function{}", signature)
    }
}
