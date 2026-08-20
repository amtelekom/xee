// The following is based on the implementation from XOT and slightly modified to work with the generic traits:
// Copyright (c) 2022 Martijn Faassen & Paligo

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::fmt::Debug;

use crate::{XmlDS, error::Error};

/// The type of the XML node.
///
/// Access it using [`Value::value_type`] or
/// [`Xot::value_type`](crate::xotdata::Xot::value_type).
///
///
/// The `ValueType` can be used if you are interested in
/// the type of the value without needing to match on it.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ValueType {
    /// Document that holds everything. Note that this not the same as the
    /// document element.
    Document,
    /// Element; it has a name, attributes and namespace information.
    Element,
    /// Text. You can get and set the text value.
    Text,
    /// Processing instruction
    ProcessingInstruction,
    /// Comment.
    Comment,
    /// Attribute
    Attribute,
    /// Namespace
    Namespace,
}

/// An XML value.
///
/// Access it using [`Xot::value`](crate::xotdata::Xot::value) or
/// mutably using [`Xot::value_mut`](crate::xotdata::Xot::value_mut).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value<S: XmlDS> {
    /// Document that holds everything. Note that this not the same as the document
    /// element.
    Document,
    /// Element; it has a name, attributes and namespace information.
    Element(S::Element),
    /// Text. You can get and set the text value.
    Text(S::Text),
    /// Processing instruction.
    ProcessingInstruction(S::ProcessingInstruction),
    /// Comment.
    Comment(S::Comment),
    /// Attribute
    Attribute(S::Attribute),
    /// Namespace
    Namespace(S::Namespace),
}

impl<S: XmlDS> Value<S> {
    /// Returns the type of the XML value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Document => ValueType::Document,
            Value::Element(_) => ValueType::Element,
            Value::Text(_) => ValueType::Text,
            Value::Comment(_) => ValueType::Comment,
            Value::ProcessingInstruction(_) => ValueType::ProcessingInstruction,
            Value::Attribute(_) => ValueType::Attribute,
            Value::Namespace(_) => ValueType::Namespace,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ValueRef<'a, S: XmlDS> {
    /// Document that holds everything. Note that this not the same as the document
    /// element.
    Document,
    /// Element; it has a name, attributes and namespace information.
    Element(&'a S::Element),
    /// Text. You can get and set the text value.
    Text(&'a S::Text),
    /// Processing instruction.
    ProcessingInstruction(&'a S::ProcessingInstruction),
    /// Comment.
    Comment(&'a S::Comment),
    /// Attribute
    Attribute(&'a S::Attribute),
    /// Namespace
    Namespace(&'a S::Namespace),
}

// /// A map of PrefixId to NamespaceId for namespace tracking.
// ///
// /// This is a real hash map, thus providing constant time access and does not
// /// preserve order information.
// ///
// /// It is used to return namespace information from various APIs.
// pub type Prefixes = AHashMap<PrefixId, NamespaceId>;

pub trait StringNewtype: Debug + PartialEq + Eq + Clone + std::hash::Hash {
    fn get(&self) -> &str;

    // fn get_mut(&mut self) -> &mut String;

    fn set<S: Into<String>>(&mut self, text: S);
}

/// XML element name.
///
/// Does not include namespace or attribute information;
/// this is kept in the tree.
///
/// Example: `<foo/>`.
pub trait Element<S: XmlDS>: Debug + PartialEq + Eq + Clone + Copy + std::hash::Hash {
    /// The name of the element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let name_doc = xot.add_name("doc");
    ///
    /// let root = xot.parse("<doc/>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element(doc_el).unwrap();
    /// assert_eq!(element.name(), name_doc);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    fn name(&self) -> S::NameId;

    /// Set the name of an element
    fn set_name(&mut self, name_id: S::NameId);
}

/// XML processing instruction value.
///
/// Example: `<?foo?>` or `<?foo bar?>`.
pub trait ProcessingInstruction<S: XmlDS>:
    Debug + Clone + Eq + PartialEq + std::hash::Hash
{
    /// Get processing instruction target.
    fn target(&self) -> S::NameId;

    /// Get processing instruction data.
    fn data(&self) -> Option<&str>;

    /// Set target
    fn set_target(&mut self, target: S::NameId) -> Result<(), Error>;

    /// Set data.
    fn set_data<V: Into<String>>(&mut self, data: Option<V>);
}

/// Represents a namespace node.
pub trait Namespace<S: XmlDS>: Debug + Clone + Eq + PartialEq + std::hash::Hash {
    /// Get prefix
    fn prefix(&self) -> S::PrefixId;

    /// Get namespace
    fn namespace(&self) -> S::NamespaceId;

    /// Set namespace id
    fn set_namespace(&mut self, namespace_id: S::NamespaceId);
}

/// Represents an attribute node.
pub trait Attribute<S: XmlDS>: Debug + Clone + Eq + PartialEq + std::hash::Hash {
    /// Get name
    fn name(&self) -> S::NameId;

    /// Get value
    fn value(&self) -> &str;

    /// Set value
    fn set_value<V: Into<String>>(&mut self, value: V);
}
