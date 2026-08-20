use xee_datasource::{Element, StringNewtype, XmlAttribAccess};
use xee_datasource::{NodeEdge, ValueRef, XmlDS};

use crate::names::Names;

pub(crate) fn strip_whitespace<S: XmlDS>(source: &mut S, names: &Names<S>, node: S::Node) {
    // all comments and processing instructions are removed
    // any text nodes that are now adjacent to each other are merged
    // we need to do this before anything else, otherwise we miss out on
    // some whitespace text nodes in the next/previous sibling rules
    strip_comment_pi(source, node);

    let mut to_remove = vec![];
    let mut xml_space_preserve = vec![];
    for edge in source.traverse(node) {
        match edge {
            NodeEdge::Start(node) => match source.value(node) {
                ValueRef::Text(text)
                    if is_xml_whitespace(text.get())
                        && !is_xml_space_preserve(source, names, node, &xml_space_preserve) =>
                {
                    to_remove.push(node);
                }
                ValueRef::Element(_) => {
                    if let Some(xml_space) = source.attributes(node).get(source.xml_space_name()) {
                        if xml_space == "preserve" {
                            xml_space_preserve.push(true);
                        } else if xml_space == "default" {
                            xml_space_preserve.push(false);
                        }
                    }
                }
                _ => {}
            },
            NodeEdge::End(node) => {
                if source.is_element(node)
                    && source
                        .attributes(node)
                        .get(source.xml_space_name())
                        .is_some()
                {
                    let _ = xml_space_preserve.pop();
                }
            }
        }
    }

    for node in to_remove {
        let _ = source.remove(node);
    }
}

fn strip_comment_pi<S: XmlDS>(source: &mut S, node: S::Node) {
    let mut to_remove = vec![];

    for node in source.descendants(node) {
        match source.value(node) {
            ValueRef::Comment(..) => {
                to_remove.push(node);
            }
            ValueRef::ProcessingInstruction(..) => {
                to_remove.push(node);
            }
            _ => {}
        }
    }

    for node in to_remove {
        let _ = source.remove(node);
    }
}

fn is_xml_space_preserve<S: XmlDS>(
    source: &S,
    names: &Names<S>,
    node: S::Node,
    xml_space_preserve: &[bool],
) -> bool {
    // if the parent is in the ignore list, we never preserve whitespace
    if let Some(parent) = source.parent(node) {
        if let Some(element) = source.element(parent) {
            // we always preserve space if the parent is xsl:text
            if element.name() == names.xsl_text {
                return true;
            }
            // we never preserve space if the parent is in the ignore list
            if names.ignore_xml_space_parents.contains(&element.name()) {
                return false;
            }
        }
    }

    if let Some(next) = source.next_sibling(node) {
        if let Some(element) = source.element(next) {
            if names
                .ignore_xml_space_next_siblings
                .contains(&element.name())
            {
                return false;
            }
        }
    }

    if let Some(previous) = source.previous_sibling(node) {
        if let Some(element) = source.element(previous) {
            if names
                .ignore_xml_space_previous_siblings
                .contains(&element.name())
            {
                return false;
            }
        }
    }

    // otherwise we look into the state of xml:space
    if let Some(last) = xml_space_preserve.last() {
        *last
    } else {
        false
    }
}

fn is_xml_whitespace_char(c: char) -> bool {
    matches!(c, '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}')
}

fn is_xml_whitespace(s: &str) -> bool {
    s.chars().all(is_xml_whitespace_char)
}

#[cfg(test)]
mod tests {
    use xot::Xot;

    use super::*;

    #[test]
    fn test_remove_comments() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><!--comment--></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc/>");
    }

    #[test]
    fn test_remove_processing_instructions() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><p>A<?pi?>B</p></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc><p>AB</p></doc>");
    }

    #[test]
    fn test_remove_whitespace() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><p>   </p></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc><p/></doc>");
    }

    // we expect this test to panic, as XML 1.0 doesn't support form feed
    #[test]
    #[should_panic]
    fn test_form_feed_is_not_whitespace() {
        // rust defines is_ascii_whitespace to include form feed, but
        // XML does not
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        // this will panic, as XML 1.0 does not support form feed
        let root = xot.parse("<doc><p>\u{0C}</p></doc>").unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc><p>\u{0C}</p></doc>");
    }

    #[test]
    fn test_whitespace_is_not_removed_inside_xsl_text() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:text>   </xsl:text></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:text>   </xsl:text></doc>"#
        );
    }

    #[test]
    fn test_whitespace_is_not_removed_inside_xml_space_preserve() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc><p xml:space="preserve">   </p><p>  </p></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc><p xml:space="preserve">   </p><p/></doc>"#
        );
    }

    #[test]
    fn test_whitespace_is_not_removed_inside_xml_space_preserve_ancestor() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc><p xml:space="preserve"><span>   </span></p><p>  </p></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc><p xml:space="preserve"><span>   </span></p><p/></doc>"#
        );
    }

    #[test]
    fn test_whitespace_is_removed_if_xml_space_set_back_to_default() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc><p xml:space="preserve"><span xml:space="default">   </span><span>   </span></p></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc><p xml:space="preserve"><span xml:space="default"/><span>   </span></p></doc>"#
        );
    }

    #[test]
    fn test_xml_space_preserve_ignored_in_special_instructions() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:accumulator xml:space="preserve">   </xsl:accumulator></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:accumulator xml:space="preserve"/></doc>"#
        );
    }

    #[test]
    fn test_xml_space_preserve_ignored_before_special_instructions() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve">   <xsl:param/></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:param/></doc>"#
        );
    }

    #[test]
    fn test_xml_space_preserve_ignored_before_special_instructions_comment_breaking_up_whitespace()
    {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve">   <!--comment-->   <xsl:param/></doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:param/></doc>"#
        );
    }

    #[test]
    fn test_xml_space_preserve_ignored_after_special_instructions() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:catch/>   </doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:catch/></doc>"#
        );
    }

    #[test]
    fn test_xml_space_preserve_ignored_after_special_instructions_comment_breaking_up_whitespace() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot
            .parse(r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:catch/>   <!--comment-->   </doc>"#)
            .unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:catch/></doc>"#
        );
    }
}
