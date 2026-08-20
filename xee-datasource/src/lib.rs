use std::any::Any;
use std::hash::Hash;
use std::sync::OnceLock;

use crate::xmlname::OwnedName;
pub use crate::xmlvalue::*;
pub use ahash::AHashMap;

pub use ::xot::Axis;
pub use ::xot::Span;

mod error;
pub mod xmlvalue;
pub use error::*;
mod nodes;
pub mod xmlgraph;
pub mod xmlname;
pub use nodes::*;
pub use xmlgraph::*;

#[cfg(feature = "xot")]
mod xot;

pub mod output {
    pub use xot::output::Indentation;
    pub mod xml {
        pub use xot::output::xml::{Declaration, DocType, Parameters};
    }
    pub mod html5 {
        pub use xot::output::html5::Parameters;
    }
}

#[allow(type_alias_bounds)]
pub type Prefixes<S: XmlDS> = ahash::AHashMap<S::PrefixId, S::NamespaceId>;

/// supertrait for all the requirements we have for something to be an ...Id:
pub trait XmlId:
    Sized + PartialEq + Eq + Clone + Copy + std::fmt::Debug + Hash + Ord + Sync + Send + 'static
{
}
impl<
    Id: Sized + PartialEq + Eq + Clone + Copy + std::fmt::Debug + Hash + Ord + Sync + Send + 'static,
> XmlId for Id
{
}

/// A trait for defining a data source that can provide XML tree data
///
/// For the default xot backend, this corresponds to the [`xot::Xot`] struct
pub trait XmlDS: Clone + core::fmt::Debug + Default + Send + Sync + 'static {
    type Node: XmlNode;
    type RefName<'a>: xmlname::RefName<'a, Self>;
    type NameId: XmlId;
    type PrefixId: XmlId;
    type NamespaceId: XmlId;

    type AttribAccess<'a>: XmlAttribAccess<'a, Self>;
    type MutAttribAccess<'a>: XmlMutAttribAccess<'a, Self>;
    type NamespaceAccess<'a>: XmlNamespaceAccess<'a, Self>;
    type MutNamespaceAccess<'a>: XmlMutNamespaceAccess<'a, Self>;

    type Element: xmlvalue::Element<Self>;
    type ProcessingInstruction: xmlvalue::ProcessingInstruction<Self>;
    type Namespace: xmlvalue::Namespace<Self>;
    type Attribute: xmlvalue::Attribute<Self>;
    type Text: xmlvalue::StringNewtype;
    type Comment: xmlvalue::StringNewtype;

    type Html<'a>;

    fn new() -> Self;

    // We need to be able to store a static StaticFunctions per Backend -
    // but due to missing generic static monomorphization, each impl needs to provide a
    // separate static Box
    fn static_functions_ref() -> &'static OnceLock<Box<dyn Any + Sync + Send>>;

    // Node Creation
    fn new_document(&mut self) -> Self::Node;
    fn new_element(&mut self, name_id: Self::NameId) -> Self::Node;
    fn new_text(&mut self, text: &str) -> Self::Node;
    fn new_comment(&mut self, text: &str) -> Self::Node;
    fn new_processing_instruction(
        &mut self,
        target_id: Self::NameId,
        text: Option<&str>,
    ) -> Self::Node;
    fn new_attribute_node(&mut self, name_id: Self::NameId, value: String) -> Self::Node;
    fn new_namespace_node(
        &mut self,
        prefix_id: Self::PrefixId,
        namespace_id: Self::NamespaceId,
    ) -> Self::Node;

    // Node Navigation
    fn document_element(&self, node: Self::Node) -> Result<Self::Node, error::Error>;
    fn first_child(&self, node: Self::Node) -> Option<Self::Node>;
    fn next_sibling(&self, node: Self::Node) -> Option<Self::Node>;
    fn previous_sibling(&self, node: Self::Node) -> Option<Self::Node>;
    fn parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn root(&self, node: Self::Node) -> Self::Node;
    fn ancestors(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_;
    fn children(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_;
    fn descendants(&self, node: Self::Node) -> impl Iterator<Item = Self::Node> + '_;
    fn axis(&self, axis: Axis, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_>;

    // Node Properties and Content
    fn value<'a>(&'a self, node: Self::Node) -> ValueRef<'a, Self>;
    fn value_type(&self, node: Self::Node) -> ValueType;
    fn element(&self, node: Self::Node) -> Option<&Self::Element>;
    fn is_document(&self, node: Self::Node) -> bool;
    fn is_element(&self, node: Self::Node) -> bool;
    fn is_attribute_node(&self, node: Self::Node) -> bool;
    fn string_value(&self, node: Self::Node) -> String;

    // Names and Namespaces
    fn add_name(&mut self, name: &str) -> Self::NameId;
    fn add_name_ns(&mut self, local_name: &str, namespace_id: Self::NamespaceId) -> Self::NameId;
    fn add_namespace(&mut self, uri: &str) -> Self::NamespaceId;
    fn add_prefix(&mut self, prefix: &str) -> Self::PrefixId;
    fn name(&self, name: &str) -> Option<Self::NameId>;
    fn name_ref<'a>(
        &'a self,
        name_id: Self::NameId,
        context: Self::Node,
    ) -> Result<Self::RefName<'a>, Error>;
    fn node_name(&self, node: Self::Node) -> Option<Self::NameId>;
    fn local_name_str(&self, name_id: Self::NameId) -> &str;
    fn uri_str(&self, name_id: Self::NameId) -> &str;
    fn name_ns_str(&self, name_id: Self::NameId) -> (&str, &str);
    fn prefix_str(&self, prefix_id: Self::PrefixId) -> &str;
    fn namespace_str(&self, namespace_id: Self::NamespaceId) -> &str;
    fn namespace_for_name(&self, name_id: Self::NameId) -> Self::NamespaceId;
    fn xml_namespace(&self) -> Self::NamespaceId;
    fn xml_space_name(&self) -> Self::NameId;
    fn node_name_ref(&self, node: Self::Node) -> Result<Option<Self::RefName<'_>>, error::Error>;

    // Attributes
    fn attributes<'a>(&'a self, node: Self::Node) -> Self::AttribAccess<'a>;
    fn get_attribute(&self, node: Self::Node, name_id: Self::NameId) -> Option<&str>;

    // Namespaces in Scope
    fn namespaces<'a>(&'a self, node: Self::Node) -> Self::NamespaceAccess<'a>;
    fn namespaces_in_scope(
        &self,
        node: Self::Node,
    ) -> Box<dyn Iterator<Item = (Self::PrefixId, Self::NamespaceId)> + '_>;
    fn prefixes(&self, node: Self::Node) -> ahash::AHashMap<Self::PrefixId, Self::NamespaceId>;

    // Node Manipulation
    fn clone_node(&mut self, node: Self::Node) -> Self::Node;
    fn append(&mut self, parent: Self::Node, child: Self::Node) -> Result<(), error::Error>;
    fn any_append(
        &mut self,
        parent: Self::Node,
        child: Self::Node,
    ) -> Result<Self::Node, error::Error>;
    fn remove(&mut self, node: Self::Node) -> Result<(), error::Error>;

    // Serialization
    fn to_string(&self, node: Self::Node) -> Result<String, error::Error>;

    // Special XML Features
    fn xml_id_node(&self, document: Self::Node, id: &str) -> Option<Self::Node>;

    // Document Order and Comparison
    fn child_index(&self, parent: Self::Node, child: Self::Node) -> Option<usize>;

    // Parsing methods
    fn parse(&mut self, xml: &str) -> Result<Self::Node, ParseError>;
    fn parse_fragment(&mut self, xml: &str) -> Result<Self::Node, ParseError>;

    // Serialization methods
    fn serialize_xml_string(
        &self,
        parameters: output::xml::Parameters,
        node: Self::Node,
    ) -> Result<String, error::Error>;

    // Tree traversal
    fn traverse(
        &self,
        node: Self::Node,
    ) -> Box<dyn Iterator<Item = xmlgraph::NodeEdge<Self::Node>> + '_>;

    // Mutable attributes access
    fn attributes_mut<'a>(&'a mut self, node: Self::Node) -> Self::MutAttribAccess<'a>;

    // Prefix handling
    fn empty_prefix(&self) -> Self::PrefixId;

    // Additional node type checks
    fn is_text(&self, node: Self::Node) -> bool;
    fn is_comment(&self, node: Self::Node) -> bool;
    fn is_namespace_node(&self, node: Self::Node) -> bool;
    fn is_processing_instruction(&self, node: Self::Node) -> bool;

    // Additional node access methods
    fn processing_instruction(&self, node: Self::Node) -> Option<&Self::ProcessingInstruction>;
    fn full_name(&self, node: Self::Node, name_id: Self::NameId) -> Result<String, error::Error>;
    fn name_ns(&self, local_name: &str, namespace_id: Self::NamespaceId) -> Option<Self::NameId>;

    // Mutable namespace access
    fn namespaces_mut<'a>(&'a mut self, node: Self::Node) -> Self::MutNamespaceAccess<'a>;

    // Deep equality for XPath
    fn deep_equal_xpath(
        &self,
        a: Self::Node,
        b: Self::Node,
        string_compare: impl Fn(&str, &str) -> bool,
    ) -> bool;

    // Tree traversal methods
    fn all_reverse_preorder(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_>;
    fn all_descendants(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_>;
    fn all_following(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_>;

    // Node cloning with prefixes
    fn clone_with_prefixes(&mut self, node: Self::Node) -> Self::Node;

    // Mutable text access
    fn text_mut(&mut self, node: Self::Node) -> Option<&mut Self::Text>;

    // HTML5 support
    fn html5(&mut self) -> Self::Html<'_>;

    fn prefix(&self, prefix: &str) -> Option<Self::PrefixId>;
    fn namespace(&self, namespace: &str) -> Option<Self::NamespaceId>;

    /// Create a new [`RefName`] from this owned name.
    fn owned_name_to_ref<'a>(&'a mut self, name: &OwnedName) -> Self::RefName<'a>;

    /// Create a new [`RefName`] only if names already exists in Xot.
    ///
    /// If the local name and namespace don't exist in Xot, returns None.
    ///
    /// This lets you pass in an immutable reference to `Xot`, unlike
    /// `Owned::to_ref`.
    ///
    /// Ignores prefix information - even if the prefix doesn't exist, we still
    /// get a ref name, just without a prefix. This is because the prefix
    /// information is irrelevant for comparison.
    ///
    /// Otherwise, returns None.
    fn owned_name_maybe_to_ref<'a>(&'a self, name: &OwnedName) -> Option<Self::RefName<'a>>;

    fn parse_with_span_info(&mut self, xml: &str) -> Result<(Self::Node, SpanInfo<Self>), Error>;

    fn text_content_str(&self, node: Self::Node) -> Option<&str>;
}

/// A marker trait for XeeDatasources that do NOT want to implement `xee-interpreter::function::SourceSpecificFunctions`
pub trait NoCustomXpathFn: XmlDS {}

/// A trait for defining a lightweight node identifier inside an XML tree
///
/// For the default xot backend, this corresponds to the [`xot::Node`] struct
pub trait XmlNode:
    PartialEq
    + Eq
    + std::hash::Hash
    + Clone
    + Copy
    + std::fmt::Debug
    + 'static
    + From<XmlNodeWrapper<Self>>
{
}

/// Because of orphaning rules across multiple crates, it isn't allowed to implement
/// all From<...> traits we need for the used XmlNode type as Blanket Impls.
///
/// Instead, they are implemented on XmlNodeWrapper, which in turn implements From/Into XmlNode bidirectionally
/// (XmlNode -> XmlNodeWrapper generically here, XmlNodeWrapper -> XmlNode in each Backend impl)
#[repr(transparent)]
pub struct XmlNodeWrapper<N: XmlNode>(pub N);
impl<N: XmlNode> From<N> for XmlNodeWrapper<N> {
    fn from(value: N) -> Self {
        Self(value)
    }
}
