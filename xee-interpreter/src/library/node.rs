// https://www.w3.org/TR/xpath-functions-31/#node-functions

use ahash::HashSet;
use ahash::HashSetExt;
use xee_datasource::Attribute;
use xee_datasource::Element;
use xee_datasource::Namespace;
use xee_datasource::ProcessingInstruction;
use xee_datasource::ValueRef;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::error;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::interpreter::Interpreter;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:name($arg as node()?) as xs:string", context_first)]
fn name<S: XmlDS>(interpreter: &Interpreter<S>, arg: Option<S::Node>) -> error::Result<String> {
    Ok(if let Some(node) = arg {
        let name = interpreter.source().node_name(node);
        if let Some(name) = name {
            interpreter.source().full_name(node, name)?
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    })
}

#[xpath_fn("fn:local-name($arg as node()?) as xs:string", context_first)]
fn local_name<S: XmlDS>(interpreter: &Interpreter<S>, arg: Option<S::Node>) -> String {
    if let Some(arg) = arg {
        let name = interpreter.source().node_name(arg);
        if let Some(name) = name {
            interpreter.source().local_name_str(name).to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:namespace-uri($arg as node()?) as xs:anyURI", context_first)]
fn namespace_uri<S: XmlDS>(interpreter: &Interpreter<S>, arg: Option<S::Node>) -> atomic::Atomic {
    let uri = if let Some(arg) = arg {
        let name = interpreter.source().node_name(arg);
        if let Some(name) = name {
            interpreter.source().uri_str(name).to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    atomic::Atomic::String(atomic::StringType::AnyURI, uri.into())
}

#[xpath_fn(
    "fn:lang($testlang as xs:string?, $node as node()) as xs:boolean",
    context_last
)]
fn lang<S: XmlDS>(
    interpreter: &Interpreter<S>,
    testlang: Option<&str>,
    node: S::Node,
) -> error::Result<bool> {
    let source = interpreter.source();
    let lang_name = source.name_ns("lang", source.xml_namespace());
    let test_lang = testlang.unwrap_or("");
    if let Some(lang_name) = lang_name {
        let mut lang = None;
        for node in source.ancestors(node) {
            if let Some(attribute) = source.get_attribute(node, lang_name) {
                lang = Some(attribute);
                break;
            }
        }
        if let Some(lang) = lang {
            let lang = lang.to_lowercase();
            let test_lang = test_lang.to_lowercase();
            if lang == test_lang {
                Ok(true)
            } else {
                Ok(lang.starts_with(&test_lang) && lang.chars().nth(test_lang.len()) == Some('-'))
            }
        } else {
            // no lang attribute found
            Ok(false)
        }
    } else {
        // no lang name known anywhere so cannot occur
        Ok(false)
    }
}

#[xpath_fn("fn:root($arg as node()?) as node()?", context_first)]
fn root<S: XmlDS>(interpreter: &Interpreter<S>, arg: Option<S::Node>) -> Option<S::Node> {
    arg.map(|arg| interpreter.source().root(arg))
}

#[xpath_fn("fn:has-children($node as node()?) as xs:boolean", context_first)]
fn has_children<S: XmlDS>(interpreter: &Interpreter<S>, node: Option<S::Node>) -> bool {
    if let Some(node) = node {
        interpreter.source().first_child(node).is_some()
    } else {
        false
    }
}

#[xpath_fn("fn:innermost($nodes as node()*) as node()*")]
fn innermost<S: XmlDS>(
    interpreter: &Interpreter<S>,
    nodes: impl Iterator<Item = error::Result<S::Node>>,
) -> error::Result<Vec<S::Node>> {
    let nodes: Vec<_> = nodes.collect::<error::Result<_>>()?;
    // get sequence of ancestors
    let mut ancestors = HashSet::new();
    for node in nodes.iter() {
        let mut parent_node = *node;
        // insert all parents into ancestors
        while let Some(parent) = interpreter.source().parent(parent_node) {
            ancestors.insert(parent);
            parent_node = parent;
        }
    }
    // now find all nodes that are not in ancestors
    let mut innermost = Vec::new();
    for node in nodes {
        if !ancestors.contains(&node) {
            innermost.push(node);
        }
    }
    Ok(innermost)
}

#[xpath_fn("fn:outermost($nodes as node()*) as node()*")]
fn outermost<S: XmlDS>(
    interpreter: &Interpreter<S>,
    nodes: impl Iterator<Item = error::Result<S::Node>>,
) -> error::Result<Vec<S::Node>> {
    let nodes: Vec<_> = nodes.collect::<error::Result<_>>()?;
    let node_set = nodes.iter().collect::<HashSet<_>>();
    // now find all nodes that don't have an ancestor in the set
    let mut outermost = Vec::new();
    'outer: for node in nodes.iter() {
        let mut parent_node = *node;
        // if we find an ancestor in node_set, then we don't add this node
        while let Some(parent) = interpreter.source().parent(parent_node) {
            if node_set.contains(&parent) {
                continue 'outer;
            }
            parent_node = parent;
        }
        outermost.push(*node);
    }
    Ok(outermost)
}

#[xpath_fn("fn:path($arg as node()?) as xs:string?", context_first)]
fn path<S: XmlDS>(interpreter: &Interpreter<S>, arg: Option<S::Node>) -> Option<String> {
    if let Some(node) = arg {
        if interpreter.source().is_document(node) {
            Some("/".to_string())
        } else {
            Some(path_helper(node, interpreter.source()))
        }
    } else {
        None
    }
}

fn path_helper<S: XmlDS>(node: S::Node, source: &S) -> String {
    match source.value(node) {
        ValueRef::Document => "".to_string(),
        ValueRef::Element(e) => {
            let name = e.name();
            let (local, ns) = source.name_ns_str(name);
            let position = position_by_type(node, source, |child, source| {
                if let Some(element) = source.element(child) {
                    element.name() == name
                } else {
                    false
                }
            });
            let path = parent_path(node, source);
            format!("{}/Q{{{}}}{}[{}]", path, ns, local, position)
        }
        ValueRef::Text(_) => {
            let position = position_by_type(node, source, |child, source| source.is_text(child));
            format!("{}/text()[{}]", parent_path(node, source), position)
        }
        ValueRef::Comment(_) => {
            let position = position_by_type(node, source, |child, source| source.is_comment(child));
            format!("{}/comment()[{}]", parent_path(node, source), position)
        }
        ValueRef::ProcessingInstruction(p) => {
            let target = p.target();
            let position = position_by_type(node, source, |child, source| {
                if let Some(processing_instruction) = source.processing_instruction(child) {
                    processing_instruction.target() == target
                } else {
                    false
                }
            });
            let (local, _) = source.name_ns_str(target);

            format!(
                "{}/processing-instruction({})[{}]",
                parent_path(node, source),
                local,
                position
            )
        }
        ValueRef::Attribute(attribute) => {
            let name = attribute.name();
            let (local, ns) = source.name_ns_str(name);
            let s = if ns.is_empty() {
                local.to_string()
            } else {
                format!("Q{{{}}}{}", ns, local)
            };
            format!("{}/@{}", parent_path(node, source), s)
        }
        ValueRef::Namespace(n) => {
            let prefix = n.prefix();
            let s = if source.empty_prefix() != prefix {
                source.prefix_str(prefix)
            } else {
                "*[Q{http://www.w3.org/2005/xpath-functions}local-name()=\"\"]"
            };
            format!("{}/namespace::{}", parent_path(node, source), s)
        }
    }
}

fn parent_path<S: XmlDS>(node: S::Node, source: &S) -> String {
    if let Some(parent) = source.parent(node) {
        path_helper(parent, source)
    } else {
        "Q{http://www.w3.org/2005/xpath-functions}root()".to_string()
    }
}

fn position_by_type<S: XmlDS>(
    node: S::Node,
    source: &S,
    is_type: impl Fn(S::Node, &S) -> bool,
) -> usize {
    let mut position = 1;
    let parent = source.parent(node);
    if let Some(parent) = parent {
        for child in source.children(parent) {
            if child == node {
                return position;
            }
            if is_type(child, source) {
                position += 1;
            }
        }
        unreachable!()
    } else {
        1
    }
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(name, S),
        wrap_xpath_fn!(local_name, S),
        wrap_xpath_fn!(namespace_uri, S),
        wrap_xpath_fn!(lang, S),
        wrap_xpath_fn!(root, S),
        wrap_xpath_fn!(has_children, S),
        wrap_xpath_fn!(innermost, S),
        wrap_xpath_fn!(outermost, S),
        wrap_xpath_fn!(path, S),
    ]
}
