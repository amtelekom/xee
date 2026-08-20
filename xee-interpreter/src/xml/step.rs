use xee_datasource::{xmlname::RefName, Attribute, Element, ValueRef, ValueType, XmlDS};

use xee_xpath_ast::ast;

use crate::sequence;

use super::kind_test::kind_test;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub axis: ast::Axis,
    pub node_test: ast::NodeTest,
}

pub(crate) fn resolve_step<S: XmlDS>(
    step: &Step,
    node: S::Node,
    source: &S,
) -> sequence::Sequence<S::Node> {
    let mut new_items = Vec::new();
    for axis_node in node_take_axis(&step.axis, source, node) {
        if node_test(&step.node_test, &step.axis, source, axis_node) {
            new_items.push(sequence::Item::Node(axis_node));
        }
    }
    new_items.into()
}

fn convert_axis(axis: &ast::Axis) -> xee_datasource::Axis {
    match axis {
        ast::Axis::Child => xee_datasource::Axis::Child,
        ast::Axis::Descendant => xee_datasource::Axis::Descendant,
        ast::Axis::Parent => xee_datasource::Axis::Parent,
        ast::Axis::Ancestor => xee_datasource::Axis::Ancestor,
        ast::Axis::FollowingSibling => xee_datasource::Axis::FollowingSibling,
        ast::Axis::PrecedingSibling => xee_datasource::Axis::PrecedingSibling,
        ast::Axis::Following => xee_datasource::Axis::Following,
        ast::Axis::Preceding => xee_datasource::Axis::Preceding,
        ast::Axis::DescendantOrSelf => xee_datasource::Axis::DescendantOrSelf,
        ast::Axis::AncestorOrSelf => xee_datasource::Axis::AncestorOrSelf,
        ast::Axis::Self_ => xee_datasource::Axis::Self_,
        ast::Axis::Attribute => xee_datasource::Axis::Attribute,
        ast::Axis::Namespace => unreachable!("Namespace axis should be forbidden at compile time"),
    }
}

fn node_take_axis<'a, S: XmlDS>(
    axis: &ast::Axis,
    source: &'a S,
    node: S::Node,
) -> Box<dyn Iterator<Item = S::Node> + 'a> {
    let axis = convert_axis(axis);
    source.axis(axis, node)
}

fn node_test<S: XmlDS>(
    node_test: &ast::NodeTest,
    axis: &ast::Axis,
    source: &S,
    node: S::Node,
) -> bool {
    match node_test {
        ast::NodeTest::KindTest(kt) => kind_test(kt, source, node),
        ast::NodeTest::NameTest(name_test) => {
            if source.value_type(node) != principal_node_kind(axis) {
                return false;
            }
            match name_test {
                ast::NameTest::Name(name) => {
                    let name = source.owned_name_maybe_to_ref(&name.value);
                    if let Some(name) = name {
                        let name_id: <S as XmlDS>::NameId = name.name_id();
                        match source.value(node) {
                            ValueRef::Element(element) => element.name() == name_id,
                            ValueRef::Attribute(attribute) => attribute.name() == name_id,
                            ValueRef::Text(_) => source.node_name(node) == Some(name_id),
                            _ => false,
                        }
                    } else {
                        // if name isn't present in any XML document it's certainly
                        // false
                        false
                    }
                }
                ast::NameTest::Star => true,
                ast::NameTest::LocalName(local_name) => match source.value(node) {
                    ValueRef::Element(element) => {
                        let name_id = element.name();
                        let (name_str, _) = source.name_ns_str(name_id);
                        name_str == local_name
                    }
                    ValueRef::Attribute(attribute) => {
                        source.local_name_str(attribute.name()) == local_name
                    }
                    _ => false,
                },
                ast::NameTest::Namespace(uri) => match source.value(node) {
                    ValueRef::Element(element) => {
                        let name_id = element.name();
                        let namespace_str = source.uri_str(name_id);
                        namespace_str == uri
                    }
                    ValueRef::Attribute(attribute) => {
                        let namespace_str = source.uri_str(attribute.name());
                        namespace_str == uri
                    }
                    _ => false,
                },
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrincipalNodeKind {
    Element,
    Attribute,
    Namespace,
}

fn principal_node_kind(axis: &ast::Axis) -> ValueType {
    match axis {
        ast::Axis::Attribute => ValueType::Attribute,
        ast::Axis::Namespace => ValueType::Namespace,
        _ => ValueType::Element,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xee_xpath_ast::{ast, WithSpan};
    use xot::Xot;

    fn xot_nodes_to_value(node: &[xot::Node]) -> sequence::Sequence<xot::Node> {
        node.iter()
            .map(|&node| sequence::Item::Node(node))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn test_child_axis_star() -> Result<(), xot::Error> {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc)?;
        let a = xot.first_child(doc_el).unwrap();
        let b = xot.next_sibling(a).unwrap();

        let step = Step {
            axis: ast::Axis::Child,
            node_test: ast::NodeTest::NameTest(ast::NameTest::Star),
        };
        let value = resolve_step(&step, doc_el, &xot);
        assert_eq!(value, xot_nodes_to_value(&[a, b]));
        Ok(())
    }

    #[test]
    fn test_child_axis_name() -> Result<(), xot::Error> {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc)?;
        let a = xot.first_child(doc_el).unwrap();

        let step = Step {
            axis: ast::Axis::Child,
            node_test: ast::NodeTest::NameTest(ast::NameTest::Name(
                ast::Name::name("a").with_empty_span(),
            )),
        };
        let value = resolve_step(&step, doc_el, &xot);
        assert_eq!(value, xot_nodes_to_value(&[a]));
        Ok(())
    }
}
