use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{GenericArgument, ItemFn, LitStr, PathArguments, ReturnType, Type};

use xee_datasource::xmlname::NameStrInfo;
use xee_xpath_ast::ast::Signature;

use crate::convert::convert_sequence_type;
use crate::parse::XPathFnOptions;

/// Create the XPath function wrapper
///
/// `specific_xmlds`: Whether this function is generic over all XmlDS implementers (= None),
/// usually all implementations in xee-interpreter use this,
/// or if this function is implemented for a specific XmlDS, and if so, what it's Identifier is
pub(crate) fn xpath_fn_wrapper(
    ast: &ItemFn,
    options: &XPathFnOptions,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &ast.sig.ident;
    let wrapper_name = format_ident!("wrapper_{}", name);

    let interpreter_crate = match proc_macro_crate::crate_name("xee-interpreter")
        .expect("Need to depend on xee-interpreter to use xee-xpath-macros!")
    {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(n) => {
            let n = Ident::new(&n, Span::call_site());
            quote! { ::#n }
        }
    };

    let wrapper = make_wrapper(
        name,
        &wrapper_name,
        ast,
        &options.signature,
        &interpreter_crate,
        &options.xmlds,
    )?;

    let vis = &ast.vis;
    let signature_string = LitStr::new(&options.signature_string, Span::call_site());
    let kind = if let Some(kind) = &options.kind {
        LitStr::new(kind, Span::call_site())
    } else {
        LitStr::new("", Span::call_site())
    };
    let impl_on_ds = if let Some(ds) = &options.xmlds {
        // if this is only implemented on a specific Datasource, just impl using that
        quote! {
            impl #name::Wrapper<#ds> {
                const WRAPPER: #interpreter_crate::function::StaticFunctionType<#ds> = #wrapper_name;
            }
        }
    } else {
        // else, create a generic impl:
        quote! {
            impl<S: ::xee_datasource::XmlDS + #interpreter_crate::function::SourceSpecificFunctions> #name::Wrapper<S> {
                const WRAPPER: #interpreter_crate::function::StaticFunctionType<S> = #wrapper_name;
            }
        }
    };
    Ok(quote! {
        // create a module with the same name as the function - this way `use
        // <the function> will bring both the function and module into scope.
        // This module contains information about the wrapper function
        // we access with the wrap_xpath_fn! macro.
        #[doc(hidden)]
        #vis mod #name {
            pub(crate) struct Wrapper<S: ::xee_datasource::XmlDS + #interpreter_crate::function::SourceSpecificFunctions> {
                _phantom: std::marker::PhantomData<S>,
            }
            // We store the signature as a string; this means we need to
            // reparse it again later during registration, but it's a lot
            // easier than trying to serialize a data structure, so it will
            // do for now.
            pub(crate) const SIGNATURE: &str = #signature_string;
            pub(crate) const KIND: &str = #kind;
        }

        // Generate the function inside of the same scope at the original
        // function (but in an isolated block), so that it can easily call the
        // original function. Using `super` isn't useful for that, as the
        // original function may be inside of a function body.
        const _: () = {
            // This is a trick to ensure we can get it into the module defined
            // above, that also helps with having a WRAPPER per XmlDS impl generic type parameter
            #impl_on_ds

            #vis #wrapper
        };
    })
}

fn make_wrapper(
    name: &Ident,
    wrapper_name: &Ident,
    ast: &ItemFn,
    signature: &Signature,
    interpreter_crate: &TokenStream,
    xmlds: &Option<Ident>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut conversions = Vec::new();
    let mut conversion_names = Vec::new();
    let mut adjust = 0;
    let context_ident = get_argument_ident(ast, adjust, "context")?;
    if let Some(context_ident) = context_ident {
        conversion_names.push(context_ident);
        adjust += 1
    }
    let interpreter_ident = get_argument_ident(ast, adjust, "interpreter")?;
    if let Some(interpreter_ident) = interpreter_ident {
        conversion_names.push(interpreter_ident);
        adjust += 1;
    }
    for (i, param) in signature.params.iter().enumerate() {
        let name = Ident::new(param.name.local_name(), Span::call_site());
        conversion_names.push(name.clone());
        let arg = quote!(arguments[#i]);
        let fn_arg = &ast.sig.inputs[i + adjust];
        conversions.push(convert_sequence_type(
            &param.type_,
            fn_arg,
            name.to_token_stream(),
            arg,
            interpreter_crate,
        )?);
    }

    let rti = eval_return_type(ast);

    // Due to generics stuff, we cannot use From/Into for everything,
    // so for some cases we need an explicit type conversion:
    let return_conv = if rti.is_xmlnode {
        if rti.is_option {
            quote!(value.map(::xee_datasource::XmlNodeWrapper).into())
        } else if rti.is_vec {
            quote!(#interpreter_crate::sequence::Sequence::new(
                value
                    .into_iter()
                    .map(::xee_datasource::XmlNodeWrapper)
                    .map(#interpreter_crate::sequence::Item::from)
                    .collect::<Vec<_>>()
            ))
        } else {
            quote!(::xee_datasource::XmlNodeWrapper(value).into())
        }
    } else {
        quote!(value.into())
    };

    let body = if rti.is_result {
        quote!(#(#conversions)*;
        let value = #name(#(#conversion_names),*);
        value.map(|value| #return_conv))
    } else {
        quote!(#(#conversions)*;
        let value = #name(#(#conversion_names),*);
        Ok(#return_conv))
    };

    if let Some(ds) = &xmlds {
        Ok(quote!(
            fn #wrapper_name(
                context: &#interpreter_crate::context::DynamicContext<#ds>,
                interpreter: &mut #interpreter_crate::interpreter::Interpreter<#ds>,
                arguments: &[#interpreter_crate::sequence::Sequence<<#ds as ::xee_datasource::XmlDS>::Node>]
            ) -> Result<#interpreter_crate::sequence::Sequence<<#ds as ::xee_datasource::XmlDS>::Node>, #interpreter_crate::error::Error> {
            #body
        }))
    } else {
        Ok(quote!(
            fn #wrapper_name<S: ::xee_datasource::XmlDS + #interpreter_crate::function::SourceSpecificFunctions>(
                context: &#interpreter_crate::context::DynamicContext<S>,
                interpreter: &mut #interpreter_crate::interpreter::Interpreter<S>,
                arguments: &[#interpreter_crate::sequence::Sequence<S::Node>]
            ) -> Result<#interpreter_crate::sequence::Sequence<S::Node>, #interpreter_crate::error::Error> {
            #body
        }))
    }
}

fn get_argument_ident(ast: &ItemFn, index: usize, name: &str) -> syn::Result<Option<Ident>> {
    if index >= ast.sig.inputs.len() {
        return Ok(None);
    }

    if !ast.sig.inputs.is_empty() {
        let maybe_context_arg = &ast.sig.inputs[index];
        match &maybe_context_arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => Ok(if ident.ident == name {
                    Some(ident.ident.clone())
                } else {
                    None
                }),
                _ => {
                    bail_spanned!(pat_type.span() => "XPath functions can only take identifiers as arguments");
                }
            },
            syn::FnArg::Receiver(r) => {
                bail_spanned!(r.span() => "XPath functions cannot take `self` as an argument");
            }
        }
    } else {
        Ok(None)
    }
}

#[derive(Default)]
struct ReturnTypeInfo {
    /// true if the funtion returns any `-> Result<_>`
    is_result: bool,
    /// true if the function returns `-> Option<_>` or `-> Result<Option<_>>`
    is_option: bool,
    /// true if the function returns `-> Vec<_>` (or wrapped in Result/Option)
    is_vec: bool,
    /// true if the function returns something that looks like an XmlNode
    is_xmlnode: bool,
}

/// Checks if a type is a specific generic wrapper (Result, Option),
/// and returns the "inner" type (or the original if not wrapped)
fn is_wrapped_in<'a>(type_: &'a Type, wrapper_name: &str) -> (bool, &'a Type) {
    match type_ {
        Type::Path(type_path) => {
            let last_seg = type_path.path.segments.last().unwrap();
            let is_wrapped = last_seg.ident.to_string().as_str() == wrapper_name;

            let inner_type = if is_wrapped {
                if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                    if let GenericArgument::Type(inner_type) = args.args.first().unwrap() {
                        inner_type
                    } else {
                        unreachable!("{wrapper_name} doesn't have an inner type...?")
                    }
                } else {
                    unreachable!("{wrapper_name} doesn't have an inner type...?")
                }
            } else {
                // there is no result, so the return type is the "inner" return
                type_
            };

            (is_wrapped, inner_type)
        }
        _ => (false, type_),
    }
}

fn eval_return_type(ast: &ItemFn) -> ReturnTypeInfo {
    let return_type = &ast.sig.output;
    match return_type {
        ReturnType::Default => ReturnTypeInfo::default(),
        ReturnType::Type(_, type_) => {
            let (is_result, type_) = is_wrapped_in(type_.as_ref(), "Result");
            let (is_option, type_) = is_wrapped_in(type_, "Option");
            let (is_vec, type_) = is_wrapped_in(type_, "Vec");
            let is_xmlnode = if let Type::Path(type_path) = type_ {
                let segs: Vec<_> = type_path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                const XMLNODE_1: [&str; 2] = ["S", "Node"];
                const XMLNODE_2: [&str; 1] = ["N"];
                segs == XMLNODE_1 || segs == XMLNODE_2
            } else {
                false
            };

            ReturnTypeInfo {
                is_result,
                is_option,
                is_vec,
                is_xmlnode,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use syn::parse_str;

    #[test]
    fn test_wrapper() {
        let options =
            parse_str::<XPathFnOptions>(r#""fn:foo($x as xs:int) as xs:string""#).unwrap();
        let ast = parse_str::<ItemFn>(
            r#"
            fn foo(x: &i64) -> String {
                format!("{}", x)
            }"#,
        )
        .unwrap();
        assert_debug_snapshot!(xpath_fn_wrapper(&ast, &options).unwrap().to_string());
    }

    #[test]
    fn test_wrapper_items_sequence_arg() {
        let options =
            parse_str::<XPathFnOptions>(r#""fn:foo($x as item()*) as xs:string""#).unwrap();
        let ast = parse_str::<ItemFn>(
            r#"
            fn foo(x: &crate::Sequence) -> String {
                "foo".to_string()
            }"#,
        )
        .unwrap();
        assert_debug_snapshot!(xpath_fn_wrapper(&ast, &options).unwrap().to_string());
    }
}
