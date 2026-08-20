use xee_datasource::Attribute;
use xee_datasource::Namespace;
use xee_datasource::StringNewtype;
use xee_datasource::ValueRef;
use xee_datasource::XmlNode;
use xee_datasource::XmlNodeWrapper;

use crate::atomic;
use crate::context;
use crate::error;
use crate::function;
use crate::function::XmlDS;

/// An XPath item. These are the items that make up an XPath sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<N: XmlNode> {
    /// An atomic value.
    ///
    /// One of the value types defined by XPath, indicated by an `xs:*` type
    /// such as `xs:integer` or `xs:string`.
    Atomic(atomic::Atomic),
    /// A node in an XML document.
    ///
    /// This is defined using the [`xot`] library.
    Node(N),
    /// An XPath function type.
    Function(function::Function<N>),
}

impl<N: XmlNode> Item<N> {
    /// Try to get the atomic value of the item.
    pub fn to_atomic(&self) -> error::Result<atomic::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(error::Error::XPTY0004(
                "Item::to_atomic called on a non-Atomic Item",
            )),
        }
    }

    /// Try to get the node value of the item.
    pub fn to_node(&self) -> error::Result<N> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(error::Error::XPTY0004(
                "Item::to_node called on a non-Node Item",
            )),
        }
    }

    /// Try to get the function value of the item.
    pub fn to_function(&self) -> error::Result<function::Function<N>> {
        match self {
            Item::Function(f) => Ok(f.clone()),
            _ => Err(error::Error::XPTY0004(
                "Item::to_function called on a non-Function Item",
            )),
        }
    }

    /// Try to get the value as an XPath Map.
    pub fn to_map(&self) -> error::Result<function::Map<N>> {
        if let Item::Function(function::Function::Map(map)) = self {
            Ok(map.clone())
        } else {
            Err(error::Error::XPTY0004(
                "Item::to_map called on an Item that is not a Map",
            ))
        }
    }

    /// Try to get the value as an XPath Array.
    pub fn to_array(&self) -> error::Result<function::Array<N>> {
        if let Item::Function(function::Function::Array(array)) = self {
            Ok(array.clone())
        } else {
            Err(error::Error::XPTY0004(
                "Item::to_array called on an Item that is not an Array",
            ))
        }
    }

    /// Obtain the [effective boolean
    /// value](https://www.w3.org/TR/xpath-31/#id-ebv) of the item.
    ///
    /// - If the item is a node, it's true.
    ///
    /// - If the item is a boolean, it's the value of the boolean.
    ///
    /// - If the item is a string, it's false if it's empty, otherwise true.
    ///
    /// - If the item is a numeric type, it's false if it's NaN or zero,
    ///   otherwise true.
    ///
    /// - Functions are always errors.
    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Item::Atomic(a) => a.effective_boolean_value(),
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Item::Node(_) => Ok(true),
            Item::Function(_) => Err(error::Error::FORG0006),
        }
    }

    /// Convert an atomic value into a value of type `V`.
    pub fn try_into_value<V>(&self) -> error::Result<V>
    where
        V: TryFrom<atomic::Atomic, Error = error::Error>,
    {
        match self {
            Item::Atomic(a) => a.clone().try_into(),
            // atomic::Atomic::try_from(a.clone()),
            _ => Err(error::Error::XPTY0004(
                "Item::try_into_value called on a non-Atomic Item",
            )),
        }
    }

    /// Construct the string value.
    ///
    /// - For an atomic value, it casts it to a string using the canonical
    ///   lexical representation rules as defined by XML Schema.
    ///
    /// - For a node, it returns the [string value of the
    ///   node](https://www.w3.org/TR/xpath-31/#id-typed-value).
    ///
    /// - For a function, it errors.
    pub fn string_value<S: XmlDS<Node = N>>(&self, source: &S) -> error::Result<String> {
        match self {
            Item::Atomic(atomic) => Ok(atomic.string_value()),
            Item::Node(node) => Ok(source.string_value(*node)),
            Item::Function(_) => Err(error::Error::FOTY0014),
        }
    }

    /// Display representation of an item
    /// For atomics this is a true parseable XPath representation.
    /// For node and function that does not exist, so we generate a plausible
    /// version for display purposes only.
    pub fn display_representation<S: XmlDS<Node = N>>(
        &self,
        source: &S,
        context: &context::DynamicContext<S>,
    ) -> error::Result<String> {
        match self {
            Item::Atomic(atomic) => Ok(atomic.xpath_representation()),
            Item::Node(node) => node_display_representation(*node, source),
            Item::Function(function) => Ok(function.display_representation(source, context)),
        }
    }

    /// Check whether this item is represents an XPath Map.
    pub(crate) fn is_map(&self) -> bool {
        match self {
            Item::Function(function) => matches!(function, function::Function::Map(_)),
            _ => false,
        }
    }

    /// Check whether this item is represents an XPath Array.
    pub(crate) fn is_array(&self) -> bool {
        match self {
            Item::Function(function) => matches!(function, function::Function::Array(_)),
            _ => false,
        }
    }
}

fn node_display_representation<S: XmlDS>(node: S::Node, source: &S) -> error::Result<String> {
    match source.value(node) {
        ValueRef::Attribute(attribute) => {
            let value = attribute.value();
            let (name, namespace) = source.name_ns_str(attribute.name());
            let name = if !namespace.is_empty() {
                format!("Q{{{}}}{}", namespace, name)
            } else {
                name.to_string()
            };
            Ok(format!("Attribute {}=\"{}\"", name, value))
        }
        ValueRef::Namespace(namespace) => {
            let prefix_id = namespace.prefix();
            let namespace_id = namespace.namespace();
            let prefix_str = source.prefix_str(prefix_id);
            let namespace_str = source.namespace_str(namespace_id);
            Ok(format!("Namespace {}:{}", prefix_str, namespace_str))
        }
        ValueRef::Text(text) => Ok(format!("Text \"{}\"", text.get())),
        // for everything else we can just serialize the node
        _ => Ok(source.serialize_xml_string(
            {
                xee_datasource::output::xml::Parameters {
                    indentation: Default::default(),
                    ..Default::default()
                }
            },
            node,
        )?),
    }
}

impl<T, N: XmlNode> From<T> for Item<N>
where
    T: Into<atomic::Atomic>,
{
    fn from(a: T) -> Self {
        Self::Atomic(a.into())
    }
}

impl<N: XmlNode> TryFrom<Item<N>> for atomic::Atomic {
    type Error = error::Error;

    fn try_from(item: Item<N>) -> error::Result<atomic::Atomic> {
        match item {
            Item::Atomic(a) => Ok(a),
            _ => Err(error::Error::XPTY0004(
                "Atomic::try_from called with a non-Atomic Item",
            )),
        }
    }
}

impl<N: XmlNode> TryFrom<&Item<N>> for atomic::Atomic {
    type Error = error::Error;

    fn try_from(item: &Item<N>) -> error::Result<atomic::Atomic> {
        match item {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(error::Error::XPTY0004(
                "Atomic::try_from called with a non-Atomic Item",
            )),
        }
    }
}

impl<N: XmlNode> From<XmlNodeWrapper<N>> for Item<N> {
    fn from(node: XmlNodeWrapper<N>) -> Self {
        Self::Node(node.0)
    }
}

// impl<N: XmlNode> TryFrom<Item<N>> for N {
//     type Error = error::Error;

//     fn try_from(item: Item<N>) -> error::Result<Self> {
//         match item {
//             Item::Node(node) => Ok(node),
//             _ => Err(error::Error::XPTY0004),
//         }
//     }
// }

impl<N: XmlNode> TryInto<XmlNodeWrapper<N>> for Item<N> {
    type Error = error::Error;

    fn try_into(self) -> Result<XmlNodeWrapper<N>, Self::Error> {
        match self {
            Item::Node(node) => Ok(XmlNodeWrapper(node)),
            _ => Err(error::Error::XPTY0004(
                "XmlNodeWrapper::try_from called with a non-Node Item",
            )),
        }
    }
}

// impl<N: XmlNode> TryFrom<&Item<N>> for N {
//     type Error = error::Error;

//     fn try_from(item: &Item<N>) -> error::Result<Self> {
//         match item {
//             Item::Node(node) => Ok(*node),
//             _ => Err(error::Error::XPTY0004),
//         }
//     }
// }

impl<N: XmlNode> TryInto<XmlNodeWrapper<N>> for &Item<N> {
    type Error = error::Error;

    fn try_into(self) -> Result<XmlNodeWrapper<N>, Self::Error> {
        match self {
            Item::Node(node) => Ok(XmlNodeWrapper(*node)),
            _ => Err(error::Error::XPTY0004(
                "XmlNodeWrapper::try_from called with a non-Node Item",
            )),
        }
    }
}

impl<N: XmlNode> TryFrom<Item<N>> for function::Function<N> {
    type Error = error::Error;

    fn try_from(item: Item<N>) -> error::Result<Self> {
        match item {
            Item::Function(f) => Ok(f.clone()),
            _ => Err(error::Error::XPTY0004(
                "Function::try_from called with a non-Function Item",
            )),
        }
    }
}

impl<N: XmlNode> TryFrom<&Item<N>> for function::Function<N> {
    type Error = error::Error;

    fn try_from(item: &Item<N>) -> error::Result<Self> {
        match item {
            Item::Function(f) => Ok(f.clone()),
            _ => Err(error::Error::XPTY0004(
                "Function::try_from called with a non-Function Item",
            )),
        }
    }
}

impl<N: XmlNode> From<function::Function<N>> for Item<N> {
    fn from(f: function::Function<N>) -> Self {
        Self::Function(f)
    }
}

impl<N: XmlNode> From<function::Array<N>> for Item<N> {
    fn from(array: function::Array<N>) -> Self {
        Self::Function(function::Function::Array(array))
    }
}

impl<N: XmlNode> From<function::Map<N>> for Item<N> {
    fn from(map: function::Map<N>) -> Self {
        Self::Function(function::Function::Map(map))
    }
}

pub enum AtomizedItemIter<'a, S: XmlDS> {
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(std::iter::Once<atomic::Atomic>),
    Array(AtomizedArrayIter<'a, S>),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a, S: XmlDS> AtomizedItemIter<'a, S> {
    pub(crate) fn new(item: &'a Item<S::Node>, source: &'a S) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a.clone())),
            Item::Node(n) => {
                let s = source.string_value(*n);
                let value = atomic::Atomic::Untyped(s.into());
                Self::Node(std::iter::once(value))
            }
            Item::Function(function) => match function {
                function::Function::Array(a) => Self::Array(AtomizedArrayIter::new(a, source)),
                _ => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
            },
        }
    }
}

impl<S: XmlDS> Iterator for AtomizedItemIter<'_, S> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Array(iter) => iter.next(),
            Self::Erroring(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Atomic(iter) => iter.size_hint(),
            Self::Node(iter) => iter.size_hint(),
            Self::Array(iter) => iter.size_hint(),
            Self::Erroring(iter) => iter.size_hint(),
        }
    }
}

// This complex atomized node iter is overkill, until such time when
// a node can deliver more than atomic value.
// #[derive(Debug, Clone)]
// pub struct AtomizedNodeIter {
//     typed_value: Vec<atomic::Atomic>,
//     typed_value_index: usize,
// }

// impl AtomizedNodeIter {
//     fn new(node: Node, xot: &Xot) -> Self {
//         Self {
//             typed_value: typed_value(xot, node),
//             typed_value_index: 0,
//         }
//     }
// }

// fn typed_value(xot: &Xot, node: Node) -> Vec<atomic::Atomic> {
//     // for now we don't know any types of nodes yet; everything is untyped
//     let s = xot.string_value(node);
//     vec![atomic::Atomic::Untyped(s.into())]
// }

// impl Iterator for AtomizedNodeIter {
//     type Item = atomic::Atomic;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.typed_value_index < self.typed_value.len() {
//             let item = self.typed_value[self.typed_value_index].clone();
//             self.typed_value_index += 1;
//             Some(item)
//         } else {
//             None
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let remaining = self.typed_value.len() - self.typed_value_index;
//         (remaining, Some(remaining))
//     }
// }

pub struct AtomizedArrayIter<'a, S: XmlDS> {
    source: &'a S,
    array: &'a function::Array<S::Node>,
    array_index: usize,
    iter: Option<Box<dyn Iterator<Item = error::Result<atomic::Atomic>> + 'a>>,
}

impl<'a, S: XmlDS> AtomizedArrayIter<'a, S> {
    fn new(array: &'a function::Array<S::Node>, source: &'a S) -> Self {
        Self {
            source,
            array,
            array_index: 0,
            iter: None,
        }
    }
}

impl<S: XmlDS> Iterator for AtomizedArrayIter<'_, S> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if there there are any more atoms in this array entry,
            // supply those
            if let Some(iter) = &mut self.iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                } else {
                    self.iter = None;
                }
            }
            let array = &self.array.0;
            // if we're at the end of the array, we're done
            if self.array_index >= array.len() {
                return None;
            }
            let sequence = &array[self.array_index];
            self.array_index += 1;

            self.iter = Some(Box::new(sequence.atomized(self.source)));
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // a very low lower bound, but it's the best we can do
        // we could potentially improve this by adding all the lens of the sequences
        // and then subtracing the lens of the sequences we have seen so far, but
        // this is a more involved calculation that touches a lot of places in memory
        let remaining = self.array.0.len() - self.array_index;
        (remaining, None)
    }
}

#[cfg(test)]
mod tests {

    // a static assertion to ensure that Item never grows in size
    #[cfg(target_arch = "x86_64")]
    static_assertions::assert_eq_size!(super::Item<::xot::Node>, [u8; 24]);
}
