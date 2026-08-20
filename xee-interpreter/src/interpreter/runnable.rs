use std::rc::Rc;

use ibig::ibig;
use iri_string::types::IriReferenceStr;

use crate::context::DocumentsRef;
use crate::context::DynamicContext;
use crate::context::StaticContext;
use crate::error::SpannedError;
use crate::function::Function;
use crate::function::XmlDS;
use crate::interpreter::interpret::ContextInfo;
use crate::sequence;
use crate::stack;
use crate::{error, string};

use super::program::FunctionInfo;
use super::Interpreter;
use super::Program;

#[derive(Debug)]
pub struct Runnable<'a, S: XmlDS> {
    program: &'a Program<S>,
    // TODO: this should be private, but is needed right now
    // to implement call_static without lifetime issues.
    // We could possibly obtain context from the interpreter directly,
    // but this leads to lifetime issues right now.
    pub(crate) dynamic_context: &'a DynamicContext<'a, S>,
}

impl<'a, S: XmlDS> Runnable<'a, S> {
    pub(crate) fn new(program: &'a Program<S>, dynamic_context: &'a DynamicContext<'a, S>) -> Self {
        Self {
            program,
            dynamic_context,
        }
    }

    fn run_value(&self, source: &'a mut S) -> error::SpannedResult<stack::Value<S::Node>> {
        let arguments = self.dynamic_context.arguments().unwrap();
        let mut interpreter = Interpreter::new(self, source);

        let context_info = if let Some(context_item) = self.dynamic_context.context_item() {
            ContextInfo {
                item: context_item.clone().into(),
                position: ibig!(1).into(),
                size: ibig!(1).into(),
            }
        } else {
            ContextInfo {
                item: stack::Value::Absent,
                position: stack::Value::Absent,
                size: stack::Value::Absent,
            }
        };

        interpreter.start(context_info, arguments);
        interpreter.run(0)?;

        let state = interpreter.state();
        // the stack has to be 1 values and return the result of the expression
        // why 1 value if the context item is on the top of the stack? This is because
        // the outer main function will pop the context item; this code is there to
        // remove the function id from the stack but the main function has no function id
        assert_eq!(
            state.stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            state.stack()
        );
        let value = state.stack().last().unwrap().clone();
        match value {
            stack::Value::Absent => Err(SpannedError {
                error: error::Error::XPDY0002,
                span: Some(self.program.span().into()),
            }),
            _ => Ok(value),
        }
    }

    /// Run the program against a sequence item.
    pub fn many(&self, source: &'a mut S) -> error::SpannedResult<sequence::Sequence<S::Node>> {
        Ok(self.run_value(source)?.try_into()?)
    }

    /// Run the program, expect a single item as the result.
    pub fn one(&self, source: &'a mut S) -> error::SpannedResult<sequence::Item<S::Node>> {
        let sequence = self.many(source)?;
        sequence.one().map_err(|error| SpannedError {
            error,
            span: Some(self.program.span().into()),
        })
    }

    /// Run the program, expect an optional single item as the result.
    pub fn option(
        &self,
        source: &'a mut S,
    ) -> error::SpannedResult<Option<sequence::Item<S::Node>>> {
        let sequence = self.many(source)?;
        let items = sequence.iter();
        sequence::option(items).map_err(|error| SpannedError {
            error,
            span: Some(self.program.span().into()),
        })
    }

    pub(crate) fn program(&self) -> &'a Program<S> {
        self.program
    }

    pub fn dynamic_context(&self) -> &'a DynamicContext<'_, S> {
        self.dynamic_context
    }

    pub fn documents(&self) -> DocumentsRef<S> {
        self.dynamic_context.documents()
    }

    pub fn static_context(&self) -> &StaticContext<S> {
        self.program.static_context()
    }

    pub fn default_collation_uri(&self) -> &IriReferenceStr {
        self.dynamic_context
            .static_context()
            .default_collation_uri()
    }

    pub fn default_collation(&self) -> error::Result<Rc<string::Collation>> {
        self.dynamic_context.static_context().default_collation()
    }

    pub fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.dynamic_context.implicit_timezone()
    }

    pub fn function_info<'b>(&self, function: &'b Function<S::Node>) -> FunctionInfo<'a, 'b, S> {
        self.program.function_info(function)
    }
}
