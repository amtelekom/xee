use std::{any::Any, borrow::Cow, mem, sync::OnceLock};

use crate::*;

impl super::XmlDS for ::xot::Xot {
    type Node = ::xot::Node;
    type RefName<'a> = ::xot::xmlname::RefName<'a>;
    type NameId = ::xot::NameId;
    type NamespaceId = ::xot::NamespaceId;
    type PrefixId = ::xot::PrefixId;
    type AttribAccess<'a> = ::xot::Attributes<'a>;
    type MutAttribAccess<'a> = ::xot::MutableAttributes<'a>;
    type NamespaceAccess<'a> = ::xot::Namespaces<'a>;
    type MutNamespaceAccess<'a> = ::xot::MutableNamespaces<'a>;

    type Element = ::xot::Element;
    type ProcessingInstruction = ::xot::ProcessingInstruction;
    type Namespace = ::xot::Namespace;
    type Attribute = ::xot::Attribute;
    type Text = ::xot::Text;
    type Comment = ::xot::Comment;

    type Html<'a> = ::xot::Html5<'a>;

    // Node Creation
    fn new_document(&mut self) -> Self::Node {
        self.new_document()
    }

    fn new_element(&mut self, name_id: Self::NameId) -> Self::Node {
        self.new_element(name_id)
    }

    fn new_text(&mut self, text: &str) -> Self::Node {
        self.new_text(text)
    }

    fn new_comment(&mut self, text: &str) -> Self::Node {
        self.new_comment(text)
    }

    fn new_processing_instruction(
        &mut self,
        target_id: Self::NameId,
        text: Option<&str>,
    ) -> Self::Node {
        self.new_processing_instruction(target_id, text)
    }

    fn new_attribute_node(&mut self, name_id: Self::NameId, value: String) -> Self::Node {
        self.new_attribute_node(name_id, value)
    }

    fn new_namespace_node(
        &mut self,
        prefix_id: Self::PrefixId,
        namespace_id: Self::NamespaceId,
    ) -> Self::Node {
        self.new_namespace_node(prefix_id, namespace_id)
    }

    // Node Navigation
    fn document_element(&self, node: Self::Node) -> Result<Self::Node, crate::error::Error> {
        Ok(self.document_element(node)?)
    }

    fn first_child(&self, node: Self::Node) -> Option<Self::Node> {
        self.first_child(node)
    }

    fn next_sibling(&self, node: Self::Node) -> Option<Self::Node> {
        self.next_sibling(node)
    }

    fn previous_sibling(&self, node: Self::Node) -> Option<Self::Node> {
        self.previous_sibling(node)
    }

    fn parent(&self, node: Self::Node) -> Option<Self::Node> {
        self.parent(node)
    }

    fn root(&self, node: Self::Node) -> Self::Node {
        self.root(node)
    }

    fn ancestors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.ancestors(node)
    }

    fn children(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.children(node)
    }

    fn descendants(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_ {
        self.descendants(node)
    }

    fn axis(
        &self,
        axis: super::Axis,
        node: Self::Node,
    ) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        ::xot::Xot::axis(self, axis, node)
    }

    // Node Properties and Content
    fn value<'a>(&'a self, node: Self::Node) -> ValueRef<'a, Self> {
        match self.value(node) {
            ::xot::Value::Document => ValueRef::Document,
            ::xot::Value::Element(element) => ValueRef::Element(element),
            ::xot::Value::Text(text) => ValueRef::Text(text),
            ::xot::Value::ProcessingInstruction(processing_instruction) => {
                ValueRef::ProcessingInstruction(processing_instruction)
            }
            ::xot::Value::Comment(comment) => ValueRef::Comment(comment),
            ::xot::Value::Attribute(attribute) => ValueRef::Attribute(attribute),
            ::xot::Value::Namespace(namespace) => ValueRef::Namespace(namespace),
        }
    }

    fn value_type(&self, node: Self::Node) -> ValueType {
        match self.value_type(node) {
            ::xot::ValueType::Document => ValueType::Document,
            ::xot::ValueType::Element => ValueType::Element,
            ::xot::ValueType::Text => ValueType::Text,
            ::xot::ValueType::ProcessingInstruction => ValueType::ProcessingInstruction,
            ::xot::ValueType::Comment => ValueType::Comment,
            ::xot::ValueType::Attribute => ValueType::Attribute,
            ::xot::ValueType::Namespace => ValueType::Namespace,
        }
    }

    fn element(&self, node: Self::Node) -> Option<&Self::Element> {
        self.element(node)
    }

    fn is_document(&self, node: Self::Node) -> bool {
        self.is_document(node)
    }

    fn is_element(&self, node: Self::Node) -> bool {
        self.is_element(node)
    }

    fn is_attribute_node(&self, node: Self::Node) -> bool {
        self.is_attribute_node(node)
    }

    fn string_value(&self, node: Self::Node) -> String {
        self.string_value(node)
    }

    // Names and Namespaces
    fn add_name(&mut self, name: &str) -> Self::NameId {
        self.add_name(name)
    }

    fn add_name_ns(&mut self, local_name: &str, namespace_id: Self::NamespaceId) -> Self::NameId {
        self.add_name_ns(local_name, namespace_id)
    }

    fn add_namespace(&mut self, uri: &str) -> Self::NamespaceId {
        self.add_namespace(uri)
    }

    fn add_prefix(&mut self, prefix: &str) -> Self::PrefixId {
        self.add_prefix(prefix)
    }

    fn name(&self, name: &str) -> Option<Self::NameId> {
        self.name(name)
    }

    fn node_name(&self, node: Self::Node) -> Option<Self::NameId> {
        self.node_name(node)
    }

    fn local_name_str(&self, name_id: Self::NameId) -> &str {
        self.local_name_str(name_id)
    }

    fn uri_str(&self, name_id: Self::NameId) -> &str {
        self.uri_str(name_id)
    }

    fn name_ns_str(&self, name_id: Self::NameId) -> (&str, &str) {
        self.name_ns_str(name_id)
    }

    fn prefix_str(&self, prefix_id: Self::PrefixId) -> &str {
        self.prefix_str(prefix_id)
    }

    fn namespace_str(&self, namespace_id: Self::NamespaceId) -> &str {
        self.namespace_str(namespace_id)
    }

    fn namespace_for_name(&self, name_id: Self::NameId) -> Self::NamespaceId {
        self.namespace_for_name(name_id)
    }

    // Special Namespaces
    fn xml_namespace(&self) -> Self::NamespaceId {
        self.xml_namespace()
    }

    fn xml_space_name(&self) -> Self::NameId {
        self.xml_space_name()
    }

    // Attributes
    fn attributes<'a>(&'a self, node: Self::Node) -> Self::AttribAccess<'a> {
        self.attributes(node)
    }

    fn get_attribute(&self, node: Self::Node, name_id: Self::NameId) -> Option<&str> {
        self.get_attribute(node, name_id)
    }

    // Namespaces in Scope
    fn namespaces<'a>(&'a self, node: Self::Node) -> Self::NamespaceAccess<'a> {
        self.namespaces(node)
    }

    fn namespaces_in_scope(
        &self,
        node: Self::Node,
    ) -> Box<dyn Iterator<Item = (Self::PrefixId, Self::NamespaceId)> + '_> {
        Box::new(self.namespaces_in_scope(node))
    }

    fn prefixes(&self, node: Self::Node) -> ahash::AHashMap<Self::PrefixId, Self::NamespaceId> {
        self.prefixes(node)
    }

    // Node Manipulation
    fn clone_node(&mut self, node: Self::Node) -> Self::Node {
        self.clone_node(node)
    }

    fn append(&mut self, parent: Self::Node, child: Self::Node) -> Result<(), crate::error::Error> {
        Ok(self.append(parent, child)?)
    }

    fn any_append(
        &mut self,
        parent: Self::Node,
        child: Self::Node,
    ) -> Result<Self::Node, crate::error::Error> {
        Ok(self.any_append(parent, child)?)
    }

    fn remove(&mut self, node: Self::Node) -> Result<(), crate::error::Error> {
        Ok(self.remove(node)?)
    }

    // Serialization
    fn to_string(&self, node: Self::Node) -> Result<String, crate::error::Error> {
        Ok(self.to_string(node)?)
    }

    // Special XML Features
    fn xml_id_node(&self, document: Self::Node, id: &str) -> Option<Self::Node> {
        self.xml_id_node(document, id)
    }

    // Document Order and Comparison
    fn child_index(&self, parent: Self::Node, child: Self::Node) -> Option<usize> {
        self.child_index(parent, child)
    }

    fn node_name_ref(
        &self,
        node: Self::Node,
    ) -> Result<Option<Self::RefName<'_>>, crate::error::Error> {
        Ok(self.node_name_ref(node)?)
    }

    // Parsing methods
    fn parse(&mut self, xml: &str) -> Result<Self::Node, super::ParseError> {
        Ok(self.parse(xml)?)
    }

    fn parse_fragment(&mut self, xml: &str) -> Result<Self::Node, super::ParseError> {
        Ok(self.parse_fragment(xml)?)
    }

    // Serialization methods
    fn serialize_xml_string(
        &self,
        parameters: super::output::xml::Parameters,
        node: Self::Node,
    ) -> Result<String, crate::error::Error> {
        Ok(self.serialize_xml_string(parameters, node)?)
    }

    // Tree traversal
    fn traverse(
        &self,
        node: Self::Node,
    ) -> Box<dyn Iterator<Item = super::xmlgraph::NodeEdge<Self::Node>> + '_> {
        Box::new(self.traverse(node).map(super::xmlgraph::NodeEdge::from))
    }

    // Mutable attributes access
    fn attributes_mut(&mut self, node: Self::Node) -> Self::MutAttribAccess<'_> {
        self.attributes_mut(node)
    }

    // Prefix handling
    fn empty_prefix(&self) -> Self::PrefixId {
        self.empty_prefix()
    }

    // Additional node type checks
    fn is_text(&self, node: Self::Node) -> bool {
        self.is_text(node)
    }

    fn is_comment(&self, node: Self::Node) -> bool {
        self.is_comment(node)
    }

    fn is_namespace_node(&self, node: Self::Node) -> bool {
        self.is_namespace_node(node)
    }

    fn is_processing_instruction(&self, node: Self::Node) -> bool {
        self.is_processing_instruction(node)
    }

    // Additional node access methods
    fn processing_instruction(&self, node: Self::Node) -> Option<&::xot::ProcessingInstruction> {
        self.processing_instruction(node)
    }

    fn full_name(
        &self,
        node: Self::Node,
        name_id: Self::NameId,
    ) -> Result<String, crate::error::Error> {
        Ok(self.full_name(node, name_id)?)
    }

    fn name_ns(&self, local_name: &str, namespace_id: Self::NamespaceId) -> Option<Self::NameId> {
        self.name_ns(local_name, namespace_id)
    }

    // Mutable namespace access
    fn namespaces_mut<'a>(&'a mut self, node: Self::Node) -> Self::MutNamespaceAccess<'a> {
        self.namespaces_mut(node)
    }

    // Deep equality for XPath
    fn deep_equal_xpath(
        &self,
        a: Self::Node,
        b: Self::Node,
        string_compare: impl Fn(&str, &str) -> bool,
    ) -> bool {
        self.deep_equal_xpath(a, b, string_compare)
    }

    // Tree traversal methods
    fn all_reverse_preorder(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.all_reverse_preorder(node))
    }

    fn all_descendants(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.all_descendants(node))
    }

    fn all_following(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.all_following(node))
    }

    // Node cloning with prefixes
    fn clone_with_prefixes(&mut self, node: Self::Node) -> Self::Node {
        self.clone_with_prefixes(node)
    }

    // Mutable text access
    fn text_mut(&mut self, node: Self::Node) -> Option<&mut ::xot::Text> {
        self.text_mut(node)
    }

    // HTML5 support
    fn html5(&mut self) -> ::xot::Html5<'_> {
        self.html5()
    }

    fn prefix(&self, prefix: &str) -> Option<Self::PrefixId> {
        self.prefix(prefix)
    }

    fn namespace(&self, namespace: &str) -> Option<Self::NamespaceId> {
        self.namespace(namespace)
    }

    #[inline]
    fn static_functions_ref() -> &'static std::sync::OnceLock<Box<dyn std::any::Any + Sync + Send>>
    {
        // because there is no monomorphization for static variables, this needs to be part of every impl XmlDS,
        // even if it is always the same 2 lines:
        static XOT_STATIC_FUNCTIONS: OnceLock<Box<dyn Any + Sync + Send>> = OnceLock::new();
        &XOT_STATIC_FUNCTIONS
    }

    fn owned_name_to_ref<'a>(&'a mut self, name: &OwnedName) -> Self::RefName<'a> {
        name.0.to_ref(self)
    }

    fn owned_name_maybe_to_ref<'a>(&'a self, name: &OwnedName) -> Option<Self::RefName<'a>> {
        name.0.maybe_to_ref(self)
    }

    fn parse_with_span_info(&mut self, xml: &str) -> Result<(Self::Node, SpanInfo<Self>), Error> {
        Ok(::xot::Xot::parse_with_span_info(self, xml).map(|(n, si)| (n, si.into()))?)
    }

    fn text_content_str(&self, node: Self::Node) -> Option<&str> {
        self.text_content_str(node)
    }

    fn name_ref<'a>(
        &'a self,
        name_id: Self::NameId,
        context: Self::Node,
    ) -> Result<Self::RefName<'a>, Error> {
        Ok(self.name_ref(name_id, context)?)
    }

    fn new() -> Self {
        Self::new()
    }
}

impl super::NoCustomXpathFn for ::xot::Xot {}

impl super::XmlNode for ::xot::Node {}

impl From<XmlNodeWrapper<::xot::Node>> for ::xot::Node {
    fn from(value: XmlNodeWrapper<::xot::Node>) -> Self {
        value.0
    }
}

impl<NSI: ::xot::xmlname::NameStrInfo> crate::xmlname::NameStrInfo for NSI {
    #[inline]
    fn local_name(&self) -> &str {
        ::xot::xmlname::NameStrInfo::local_name(self)
    }

    #[inline]
    fn namespace(&self) -> &str {
        ::xot::xmlname::NameStrInfo::namespace(self)
    }

    #[inline]
    fn prefix(&self) -> &str {
        ::xot::xmlname::NameStrInfo::prefix(self)
    }
}

impl<'a> crate::xmlname::RefName<'a, ::xot::Xot> for ::xot::xmlname::RefName<'a> {
    #[inline]
    fn has_unprefixed_namespace(&self) -> bool {
        ::xot::xmlname::RefName::has_unprefixed_namespace(self)
    }

    #[inline]
    fn name_id(&self) -> ::xot::NameId {
        self.name_id()
    }

    #[inline]
    fn namespace_id(&self) -> ::xot::NamespaceId {
        self.namespace_id()
    }

    #[inline]
    fn prefix_id(&self) -> ::xot::PrefixId {
        self.prefix_id()
    }
}

impl From<::xot::Error> for crate::error::Error {
    fn from(value: ::xot::Error) -> Self {
        match value {
            ::xot::Error::MissingPrefix(prefix) => crate::error::Error::MissingPrefix(prefix),
            ::xot::Error::UnknownPrefix(prefix) => crate::error::Error::UnknownPrefix(prefix),
            ::xot::Error::InvalidComment(comment) => crate::error::Error::InvalidComment(comment),
            e => crate::error::Error::Unspecified(format!("xot error: {e}")),
        }
    }
}
impl From<::xot::ParseError> for crate::error::Error {
    fn from(value: ::xot::ParseError) -> Self {
        crate::error::Error::ParseError(value.into())
    }
}
impl From<::xot::ParseError> for crate::error::ParseError {
    fn from(e: ::xot::ParseError) -> Self {
        crate::error::ParseError::Unspecified((format!("xot parse error: {e}"), e.span()))
    }
}

impl<'a> XmlNamespaceAccess<'a, ::xot::Xot> for ::xot::Namespaces<'a> {
    fn iter(&self) -> impl Iterator<Item = (::xot::PrefixId, &'a ::xot::NamespaceId)> {
        self.iter()
    }
}

impl<'a> XmlMutNamespaceAccess<'a, ::xot::Xot> for ::xot::MutableNamespaces<'a> {
    fn insert(
        &mut self,
        prefix: ::xot::PrefixId,
        namespace: ::xot::NamespaceId,
    ) -> Option<::xot::NamespaceId> {
        self.insert(prefix, namespace)
    }

    fn remove(
        &mut self,
        key: <::xot::Xot as XmlDS>::PrefixId,
    ) -> Option<<::xot::Xot as XmlDS>::NamespaceId> {
        self.remove(key)
    }
}

impl Element<::xot::Xot> for ::xot::Element {
    fn name(&self) -> ::xot::NameId {
        self.name()
    }

    fn set_name(&mut self, name_id: ::xot::NameId) {
        self.set_name(name_id);
    }
}

impl ProcessingInstruction<::xot::Xot> for ::xot::ProcessingInstruction {
    fn target(&self) -> ::xot::NameId {
        self.target()
    }

    fn data(&self) -> Option<&str> {
        self.data()
    }

    fn set_target(&mut self, target: ::xot::NameId) -> Result<(), Error> {
        Ok(self.set_target::<String>(target)?)
    }

    fn set_data<V: Into<String>>(&mut self, data: Option<V>) {
        self.set_data(data)
    }
}
impl Namespace<::xot::Xot> for ::xot::Namespace {
    fn prefix(&self) -> ::xot::PrefixId {
        self.prefix()
    }

    fn namespace(&self) -> ::xot::NamespaceId {
        self.namespace()
    }

    fn set_namespace(&mut self, namespace_id: ::xot::NamespaceId) {
        self.set_namespace(namespace_id);
    }
}
impl Attribute<::xot::Xot> for ::xot::Attribute {
    fn name(&self) -> ::xot::NameId {
        self.name()
    }

    fn value(&self) -> &str {
        self.value()
    }

    fn set_value<V: Into<String>>(&mut self, value: V) {
        self.set_value(value);
    }
}
impl StringNewtype for ::xot::Text {
    fn get(&self) -> &str {
        self.get()
    }
    fn set<S: Into<String>>(&mut self, text: S) {
        self.set(text);
    }
}
impl StringNewtype for ::xot::Comment {
    fn get(&self) -> &str {
        self.get()
    }
    fn set<S: Into<String>>(&mut self, text: S) {
        self.set(text).unwrap();
    }
}
impl<'a> XmlAttribAccess<'a, ::xot::Xot> for ::xot::Attributes<'a> {
    fn get(&self, key: ::xot::NameId) -> Option<Cow<'a, str>> {
        self.get(key).map(|v| Cow::Borrowed(v.as_str()))
    }

    fn keys(&self) -> impl Iterator<Item = <::xot::Xot as XmlDS>::NameId> + '_ {
        self.keys()
    }
}
impl<'a> XmlMutAttribAccess<'a, ::xot::Xot> for ::xot::MutableAttributes<'a> {
    fn insert(&mut self, key: ::xot::NameId, value: String) -> Option<String> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: <::xot::Xot as XmlDS>::NameId) -> Option<String> {
        self.remove(key)
    }
}

impl From<::xot::NodeEdge> for xmlgraph::NodeEdge<::xot::Node> {
    fn from(value: ::xot::NodeEdge) -> Self {
        match value {
            ::xot::NodeEdge::Start(node) => xmlgraph::NodeEdge::Start(node),
            ::xot::NodeEdge::End(node) => xmlgraph::NodeEdge::End(node),
        }
    }
}

impl From<::xot::SpanInfo> for xmlgraph::SpanInfo<::xot::Xot> {
    fn from(value: ::xot::SpanInfo) -> Self {
        // ToDo: Very ugly hack becaus this is actually undefined behavior (::xot::SpanInfo has no repr) -
        // but will need a patch in XOT to allow reading the whole Hash from the source:
        let source_hash: ahash::HashMap<::xot::SpanInfoKey, ::xot::Span> =
            unsafe { mem::transmute(value) };
        Self {
            map: source_hash
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
        }
    }
}

impl From<::xot::SpanInfoKey> for xmlgraph::SpanInfoKey<::xot::Node, ::xot::NameId> {
    fn from(value: ::xot::SpanInfoKey) -> Self {
        match value {
            ::xot::SpanInfoKey::AttributeName(node, name_id) => Self::AttributeName(node, name_id),
            ::xot::SpanInfoKey::AttributeValue(node, name_id) => {
                Self::AttributeValue(node, name_id)
            }
            ::xot::SpanInfoKey::ElementStart(node) => Self::ElementStart(node),
            ::xot::SpanInfoKey::ElementEnd(node) => Self::ElementEnd(node),
            ::xot::SpanInfoKey::Text(node) => Self::Text(node),
            ::xot::SpanInfoKey::Comment(node) => Self::Comment(node),
            ::xot::SpanInfoKey::PiTarget(node) => Self::PiTarget(node),
            ::xot::SpanInfoKey::PiContent(node) => Self::PiContent(node),
        }
    }
}
