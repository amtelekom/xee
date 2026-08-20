use xee_interpreter::function::XmlDS;
use xee_interpreter::{context, error, interpreter::Program};
use xee_ir::{compile_xpath, Variables};
use xee_xpath_ast::ast;

use crate::ast_ir::IrConverter;

/// Construct a program from an XPath AST.
pub fn compile<S: XmlDS>(
    static_context: context::StaticContext<S>,
    xpath: ast::XPath,
) -> error::SpannedResult<Program<S>> {
    let mut variables = Variables::new();
    let mut ir_converter = IrConverter::new(&mut variables, &static_context);
    let expr = ir_converter.convert_xpath(&xpath)?;
    compile_xpath(expr, static_context)
}

/// Parse an XPath string into a program.
pub fn parse<S: XmlDS>(
    static_context: context::StaticContext<S>,
    xpath: &str,
) -> error::SpannedResult<Program<S>> {
    let xpath = static_context.parse_xpath(xpath)?;
    compile(static_context, xpath)
}
