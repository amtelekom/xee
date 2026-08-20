// https://www.w3.org/TR/xslt-xquery-serialization-31/#serdm

use xee_datasource::{StringNewtype, ValueRef, XmlDS, XmlNode};

use crate::{atomic, error};

use super::{core::Sequence, item::Item};

enum NodeOrString<N: XmlNode> {
    Node(N),
    String(String),
}

pub(crate) fn normalize<S: XmlDS>(
    sequence: &Sequence<S::Node>,
    item_separator: &str,
    source: &mut S,
) -> error::Result<S::Node> {
    // 1.
    let sequence = if !sequence.is_empty() {
        // any arrays in the sequences sare flattened
        sequence.flatten()?
    } else {
        let atom: atomic::Atomic = "".into();
        Sequence::from(vec![atom])
    };
    // 2. and 3.
    let mut items: Vec<NodeOrString<S::Node>> = Vec::new();
    for item in sequence.iter() {
        match item {
            Item::Atomic(atomic) => {
                let s = atomic.clone().into_canonical();
                if let Some(NodeOrString::String(last_s)) = items.last_mut() {
                    last_s.push_str(item_separator);
                    last_s.push_str(&s);
                } else {
                    items.push(NodeOrString::String(s));
                }
            }
            Item::Node(node) => {
                items.push(NodeOrString::Node(node));
            }
            Item::Function(_) => {
                return Err(error::Error::SENR0001);
            }
        }
    }

    // 4 and 5.
    let mut flattened_nodes = Vec::new();
    for item in items {
        match item {
            NodeOrString::Node(node) => {
                // we have to clone the node here as
                // we don't want to mutate the original document
                // after this, all nodes should be cloned
                let node = source.clone_with_prefixes(node);
                if matches!(source.value(node), ValueRef::Document) {
                    for child in source.children(node) {
                        flattened_nodes.push(child);
                    }
                    continue;
                }
                flattened_nodes.push(node);
            }
            NodeOrString::String(s) => {
                let text = source.new_text(&s);
                flattened_nodes.push(text);
            }
        }
    }

    // 6. and part of 7
    let mut nodes = Vec::new();
    for node in flattened_nodes {
        match source.value(node) {
            ValueRef::Text(text) => {
                let text = text.get();
                if !text.is_empty() {
                    let text = text.to_string();
                    if let Some(last_node) = nodes.last_mut() {
                        if let Some(last_text) = source.text_mut(*last_node) {
                            let new_text = format!("{}{}", last_text.get(), text);
                            last_text.set(new_text);
                            continue;
                        }
                    }
                } else {
                    // empty text nodes are skipped
                    continue;
                }
            }
            ValueRef::Document => {
                unreachable!("Documents should have been flattened by this point");
            }
            ValueRef::Attribute(_) | ValueRef::Namespace(_) => {
                return Err(error::Error::SENR0001);
            }
            // anything else is acceptable as a node
            _ => {}
        }
        nodes.push(node);
    }

    let document = source.new_document();
    for node in nodes {
        source.append(document, node).unwrap();
    }
    Ok(document)
}
