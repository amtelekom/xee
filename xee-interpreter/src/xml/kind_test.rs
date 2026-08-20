use xee_datasource::XmlDS;
use xee_schema_type::Xs;

use xee_xpath_ast::ast;

pub(crate) fn kind_test<S: XmlDS>(kind_test: &ast::KindTest, source: &S, node: S::Node) -> bool {
    match kind_test {
        ast::KindTest::Document(dt) => document_test(dt.as_ref(), source, node),
        ast::KindTest::Element(et) => element_test(et.as_ref(), source, node),
        ast::KindTest::SchemaElement(_set) => {
            todo!()
        }
        ast::KindTest::Attribute(at) => attribute_test(at.as_ref(), source, node),
        ast::KindTest::SchemaAttribute(_sat) => {
            todo!()
        }
        ast::KindTest::Any => true,
        // text() matches any text node
        ast::KindTest::Text => source.is_text(node),
        // comment() matches any comment node
        ast::KindTest::Comment => source.is_comment(node),
        ast::KindTest::NamespaceNode => source.is_namespace_node(node),
        ast::KindTest::PI(pi_test) => {
            if !source.is_processing_instruction(node) {
                return false;
            }
            if let Some(_pi_test) = pi_test {
                // processing-instruction N matches any processing-instruction node
                // whose PITarget is equal to fn:normalize-space(N)
                // TODO
                return false;
            }
            true
        }
    }
}

fn element_test<S: XmlDS>(
    test: Option<&ast::ElementOrAttributeTest>,
    source: &S,
    node: S::Node,
) -> bool {
    element_or_attribute_test(test, source, node, |node, source| source.is_element(node))
}

fn attribute_test<S: XmlDS>(
    test: Option<&ast::ElementOrAttributeTest>,
    source: &S,
    node: S::Node,
) -> bool {
    element_or_attribute_test(test, source, node, |node, source| {
        source.is_attribute_node(node)
    })
}

fn document_test<S: XmlDS>(test: Option<&ast::DocumentTest>, source: &S, node: S::Node) -> bool {
    if !source.is_document(node) {
        return false;
    }

    if let Some(document_test) = test {
        // get document element node

        // will always succeed as node is the root node
        let document_element_node = source.document_element(node).unwrap();

        match document_test {
            ast::DocumentTest::Element(et) => {
                element_test(et.as_ref(), source, document_element_node)
            }
            ast::DocumentTest::SchemaElement(_set) => {
                todo!()
            }
        }
    } else {
        true
    }
}

fn element_or_attribute_test<S: XmlDS>(
    test: Option<&ast::ElementOrAttributeTest>,
    source: &S,
    node: S::Node,
    node_type_match: impl Fn(S::Node, &S) -> bool,
) -> bool {
    // if we're not the right node type (element, or attribute) then we
    // bail out
    if !node_type_match(node, source) {
        return false;
    }

    if let Some(test) = test {
        // the name has to match first
        let name_matches = match &test.name_or_wildcard {
            ast::NameOrWildcard::Name(name) => {
                // TODO: what if we can't find a prefix here?
                if let Some(node_name) = source.node_name_ref(node).unwrap() {
                    source.owned_name_maybe_to_ref(name) == Some(node_name)
                } else {
                    false
                }
            }
            ast::NameOrWildcard::Wildcard => true,
        };
        if !name_matches {
            return false;
        }
        // the type also has to match
        if let Some(type_name) = &test.type_name {
            type_annotation(source, node).derives_from(type_name.name)
            // ignoring can_be_nilled for now
        } else {
            true
        }
    } else {
        // there is further test, so we're done
        true
    }
}

fn type_annotation<S: XmlDS>(_source: &S, _node: S::Node) -> Xs {
    // for now we don't know any types of nodes yet
    Xs::UntypedAtomic
}

#[cfg(test)]
mod tests {
    use xee_xpath_ast::parse_kind_test;
    use xot::Xot;

    use super::*;

    #[test]
    fn test_kind_test_any() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();

        let kt = parse_kind_test("node()").unwrap();
        assert!(kind_test(&kt, &source, doc));
        assert!(kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, a));
    }

    #[test]
    fn test_kind_test_text() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>content</a><b/></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let a_text = source.first_child(a).unwrap();

        let kt = parse_kind_test("text()").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(!kind_test(&kt, &source, a));
        assert!(kind_test(&kt, &source, a_text));
    }

    #[test]
    fn test_kind_test_comment() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><!-- comment --></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let comment = source.first_child(doc_el).unwrap();

        let kt = parse_kind_test("comment()").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, comment));
    }

    #[test]
    fn test_kind_test_document() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let kt = parse_kind_test("document-node()").unwrap();
        assert!(kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
    }

    #[test]
    fn test_kind_test_element_without_name() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();

        let kt = parse_kind_test("element()").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, a));
        assert!(!kind_test(&kt, &source, text));
    }

    #[test]
    fn test_kind_test_element_with_wildcard() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();

        let kt = parse_kind_test("element(*)").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, a));
        assert!(!kind_test(&kt, &source, text));
    }

    #[test]
    fn test_kind_test_element_with_name() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();

        let kt = parse_kind_test("element(a)").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, a));
        assert!(!kind_test(&kt, &source, text));
    }

    #[test]
    fn test_kind_test_element_with_type_name() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();

        let kt = parse_kind_test("element(a, xs:untypedAtomic)").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(kind_test(&kt, &source, a));
        assert!(!kind_test(&kt, &source, text));

        // but we're not an xs:string
        let kt = parse_kind_test("element(a, xs:string)").unwrap();
        assert!(!kind_test(&kt, &source, a));
    }

    #[test]
    fn test_kind_test_attribute_without_name() {
        let mut source = Xot::new();
        let alpha = source.add_name("alpha");
        let beta = source.add_name("beta");
        let doc = source
            .parse(r#"<root><a alpha="Alpha" beta="Beta">text</a></root>"#)
            .unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();
        let alpha = source.attributes(a).get_node(alpha).unwrap();
        let beta = source.attributes(a).get_node(beta).unwrap();

        let kt = parse_kind_test("attribute()").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(!kind_test(&kt, &source, a));
        assert!(kind_test(&kt, &source, alpha));
        assert!(kind_test(&kt, &source, beta));
        assert!(!kind_test(&kt, &source, text));
    }

    #[test]
    fn test_kind_test_attribute_with_name() {
        let mut source = Xot::new();
        let alpha = source.add_name("alpha");
        let beta = source.add_name("beta");
        let doc = source
            .parse(r#"<root><a alpha="Alpha" beta="Beta">text</a></root>"#)
            .unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();
        let alpha = source.attributes(a).get_node(alpha).unwrap();
        let beta = source.attributes(a).get_node(beta).unwrap();

        let kt = parse_kind_test("attribute(alpha)").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(!kind_test(&kt, &source, a));
        assert!(kind_test(&kt, &source, alpha));
        assert!(!kind_test(&kt, &source, beta));
        assert!(!kind_test(&kt, &source, text));
    }

    #[test]
    fn test_kind_test_attribute_with_type_name() {
        let mut source = Xot::new();
        let alpha = source.add_name("alpha");
        let beta = source.add_name("beta");
        let doc = source
            .parse(r#"<root><a alpha="Alpha" beta="Beta">text</a></root>"#)
            .unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();
        let alpha = source.attributes(a).get_node(alpha).unwrap();
        let beta = source.attributes(a).get_node(beta).unwrap();

        let kt = parse_kind_test("attribute(alpha, xs:untypedAtomic)").unwrap();
        assert!(!kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(!kind_test(&kt, &source, a));
        assert!(kind_test(&kt, &source, alpha));
        assert!(!kind_test(&kt, &source, beta));
        assert!(!kind_test(&kt, &source, text));

        let kt = parse_kind_test("attribute(alpha, xs:string)").unwrap();
        assert!(!kind_test(&kt, &source, alpha));
    }

    #[test]
    fn test_kind_test_document_with_name() {
        let mut source = Xot::new();
        let doc = source.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = source.document_element(doc).unwrap();
        let a = source.first_child(doc_el).unwrap();
        let text = source.first_child(a).unwrap();

        let kt = parse_kind_test("document-node(element(root))").unwrap();
        assert!(kind_test(&kt, &source, doc));
        assert!(!kind_test(&kt, &source, doc_el));
        assert!(!kind_test(&kt, &source, a));
        assert!(!kind_test(&kt, &source, text));

        let kt = parse_kind_test("document-node(element(a))").unwrap();
        // the document doesn't match as its root node isn't 'a'
        assert!(!kind_test(&kt, &source, doc));
        // the 'a' node doesn't match either as it's not a document node
        assert!(!kind_test(&kt, &source, a));
    }
}
