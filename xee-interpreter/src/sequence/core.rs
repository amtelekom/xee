// this is unfortunately a ridiculously verbose module, wiring everything up
// carefully so we don't use performance due to dynamic dispatch on the inside.
// The verbosity is all pretty straightforward though.

// creation.rs contains various functions that create Sequence
// compare.rs contains various comparison functions

use xee_datasource::XmlNode;

use crate::function::XmlDS;
use crate::{
    atomic::{self, AtomicCompare},
    context, error, function,
    string::Collation,
    xml,
};

use super::{
    item::Item,
    traits::{BoxedItemIter, SequenceCompare, SequenceCore, SequenceExt, SequenceOrder},
    variant::{Empty, Many, One, Range},
};

// The Sequence that goes onto the stack is the size of an single item, as
// that's the biggest thing in it.
#[derive(Debug, Clone, PartialEq)]
pub enum Sequence<N: XmlNode> {
    Empty(Empty),
    One(One<N>),
    Many(Many<N>),
    Range(Range),
}

impl<N: XmlNode> From<Range> for Sequence<N> {
    fn from(inner: Range) -> Self {
        Self::Range(inner)
    }
}

impl<N: XmlNode> Default for Sequence<N> {
    fn default() -> Self {
        Self::Empty(Empty {})
    }
}

impl<N: XmlNode> Sequence<N> {
    /// Check whether the sequence is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Sequence::Empty(inner) => SequenceCore::<_, N>::is_empty(inner),
            Sequence::One(inner) => inner.is_empty(),
            Sequence::Many(inner) => inner.is_empty(),
            Sequence::Range(inner) => SequenceCore::<_, N>::is_empty(inner),
        }
    }

    /// Get the sequence length
    pub fn len(&self) -> usize {
        match self {
            Sequence::Empty(inner) => SequenceCore::<_, N>::len(inner),
            Sequence::One(inner) => inner.len(),
            Sequence::Many(inner) => inner.len(),
            Sequence::Range(inner) => SequenceCore::<_, N>::len(inner),
        }
    }

    /// Get an item in the index, if it exists
    pub fn get(&self, index: usize) -> Option<Item<N>> {
        match self {
            Sequence::Empty(inner) => inner.get(index),
            Sequence::One(inner) => inner.get(index),
            Sequence::Many(inner) => inner.get(index),
            Sequence::Range(inner) => inner.get(index),
        }
    }

    /// Get a single item from the sequence, if it only contains one item
    ///
    /// Otherwise you get a type error.
    pub fn one(self) -> error::Result<Item<N>> {
        match self {
            Sequence::Empty(inner) => inner.one(),
            Sequence::One(inner) => inner.one(),
            Sequence::Many(inner) => inner.one(),
            Sequence::Range(inner) => inner.one(),
        }
    }

    /// Get a optional item from the sequence
    ///
    /// If it contains more than one item, you get a type error.
    pub fn option(self) -> error::Result<Option<Item<N>>> {
        match self {
            Sequence::Empty(inner) => inner.option(),
            Sequence::One(inner) => inner.option(),
            Sequence::Many(inner) => inner.option(),
            Sequence::Range(inner) => inner.option(),
        }
    }

    /// Get the items from the sequence as an iterator
    pub fn iter(&self) -> BoxedItemIter<'_, N> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.iter()),
            Sequence::One(inner) => Box::new(inner.iter()),
            Sequence::Many(inner) => Box::new(inner.iter()),
            Sequence::Range(inner) => Box::new(inner.iter()),
        }
    }

    /// Effective boolean value
    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Sequence::Empty(inner) => SequenceCore::<_, N>::effective_boolean_value(inner),
            Sequence::One(inner) => inner.effective_boolean_value(),
            Sequence::Many(inner) => inner.effective_boolean_value(),
            Sequence::Range(inner) => SequenceCore::<_, N>::effective_boolean_value(inner),
        }
    }

    /// String value
    pub fn string_value<S: XmlDS<Node = N>>(&self, source: &S) -> error::Result<String> {
        match self {
            Sequence::Empty(inner) => inner.string_value(source),
            Sequence::One(inner) => inner.string_value(source),
            Sequence::Many(inner) => inner.string_value(source),
            Sequence::Range(inner) => inner.string_value(source),
        }
    }

    /// Iterator over the nodes in the sequence
    ///
    /// An error is returned for items that are not a node.
    pub fn nodes<'a>(&'a self) -> Box<dyn Iterator<Item = error::Result<N>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.nodes()),
            Sequence::One(inner) => Box::new(inner.nodes()),
            Sequence::Many(inner) => Box::new(inner.nodes()),
            Sequence::Range(inner) => Box::new(inner.nodes()),
        }
    }

    /// Iterator for the atomized values in the sequence
    pub fn atomized<'a, S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> Box<dyn Iterator<Item = error::Result<atomic::Atomic>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.atomized(source)),
            Sequence::One(inner) => Box::new(inner.atomized(source)),
            Sequence::Many(inner) => Box::new(inner.atomized(source)),
            Sequence::Range(inner) => Box::new(inner.atomized(source)),
        }
    }

    /// Get just one atomized value from the sequence
    ///
    /// If there are less or more, you get a type error.
    pub fn atomized_one<S: XmlDS<Node = N>>(&self, source: &S) -> error::Result<atomic::Atomic> {
        match self {
            Sequence::Empty(inner) => inner.atomized_one(source),
            Sequence::One(inner) => inner.atomized_one(source),
            Sequence::Many(inner) => inner.atomized_one(source),
            Sequence::Range(inner) => inner.atomized_one(source),
        }
    }

    /// Get an optional atomized value from the sequence
    ///
    /// If there are more than one, you get a type error.
    pub fn atomized_option<S: XmlDS<Node = N>>(
        &self,
        source: &S,
    ) -> error::Result<Option<atomic::Atomic>> {
        match self {
            Sequence::Empty(inner) => inner.atomized_option(source),
            Sequence::One(inner) => inner.atomized_option(source),
            Sequence::Many(inner) => inner.atomized_option(source),
            Sequence::Range(inner) => inner.atomized_option(source),
        }
    }

    /// Is used internally by the library macro.
    pub fn unboxed_atomized<'a, T: 'a, S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
        extract: impl Fn(atomic::Atomic) -> error::Result<T> + 'a,
    ) -> Box<dyn Iterator<Item = error::Result<T>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.unboxed_atomized(source, extract)),
            Sequence::One(inner) => Box::new(inner.unboxed_atomized(source, extract)),
            Sequence::Many(inner) => Box::new(inner.unboxed_atomized(source, extract)),
            Sequence::Range(inner) => Box::new(inner.unboxed_atomized(source, extract)),
        }
    }

    /// Iterator over the XPath maps in the sequence
    ///
    /// An error is returned for items that are not a map.
    pub fn map_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = error::Result<function::Map<N>>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.map_iter()),
            Sequence::One(inner) => Box::new(inner.map_iter()),
            Sequence::Many(inner) => Box::new(inner.map_iter()),
            Sequence::Range(inner) => Box::new(inner.map_iter()),
        }
    }

    /// Iterator over the XPath arrays in the sequence
    ///
    /// An error is returned for items that are not an array.
    pub fn array_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = error::Result<function::Array<N>>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.array_iter()),
            Sequence::One(inner) => Box::new(inner.array_iter()),
            Sequence::Many(inner) => Box::new(inner.array_iter()),
            Sequence::Range(inner) => Box::new(inner.array_iter()),
        }
    }

    /// Oterator over elements nodes in the sequence
    ///
    /// An error is returned for items that are not an element.
    pub fn elements<'a, S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> error::Result<Box<dyn Iterator<Item = error::Result<N>> + 'a>> {
        match self {
            Sequence::Empty(inner) => Ok(Box::new(inner.elements(source)?)),
            Sequence::One(inner) => Ok(Box::new(inner.elements(source)?)),
            Sequence::Many(inner) => Ok(Box::new(inner.elements(source)?)),
            Sequence::Range(inner) => Ok(Box::new(inner.elements(source)?)),
        }
    }

    /// Create an XPath array from this sequence.
    pub fn to_array(&self) -> error::Result<function::Array<N>> {
        match self {
            Sequence::Empty(inner) => inner.to_array(),
            Sequence::One(inner) => inner.to_array(),
            Sequence::Many(inner) => inner.to_array(),
            Sequence::Range(inner) => inner.to_array(),
        }
    }

    pub(crate) fn general_comparison<O, S: XmlDS<Node = N>>(
        &self,
        other: &Self,
        op: O,
        context: &context::DynamicContext<S>,
        source: &S,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
    {
        match (self, other) {
            (Sequence::Empty(_a), Sequence::Empty(_b)) => Ok(false),
            (Sequence::Empty(_a), Sequence::One(_b)) => Ok(false),
            (Sequence::Empty(_a), Sequence::Many(_b)) => Ok(false),
            (Sequence::Empty(_a), Sequence::Range(_b)) => Ok(false),
            (Sequence::One(_a), Sequence::Empty(_b)) => Ok(false),
            (Sequence::One(a), Sequence::One(b)) => a.general_comparison(b, op, context, source),
            (Sequence::One(a), Sequence::Many(b)) => a.general_comparison(b, op, context, source),
            (Sequence::One(a), Sequence::Range(b)) => {
                if let Item::Atomic(atomic::Atomic::Integer(_, i)) = a.item() {
                    Ok(b.general_comparison_integer(i, O::value()))
                } else {
                    a.general_comparison(b, op, context, source)
                }
            }
            (Sequence::Many(_a), Sequence::Empty(_b)) => Ok(false),
            (Sequence::Many(a), Sequence::One(b)) => a.general_comparison(b, op, context, source),
            (Sequence::Many(a), Sequence::Many(b)) => a.general_comparison(b, op, context, source),
            (Sequence::Many(a), Sequence::Range(b)) => a.general_comparison(b, op, context, source),
            (Sequence::Range(_a), Sequence::Empty(_b)) => Ok(false),
            (Sequence::Range(a), Sequence::One(b)) => a.general_comparison(b, op, context, source),
            (Sequence::Range(a), Sequence::Many(b)) => a.general_comparison(b, op, context, source),
            (Sequence::Range(a), Sequence::Range(b)) => {
                a.general_comparison(b, op, context, source)
            }
        }
    }

    pub(crate) fn value_compare<O, S: XmlDS<Node = N>>(
        &self,
        other: &Self,
        op: O,
        collation: &Collation,
        timezone: chrono::FixedOffset,
        source: &S,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
    {
        match (self, other) {
            (Sequence::Empty(a), Sequence::Empty(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Empty(a), Sequence::One(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Empty(a), Sequence::Many(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Empty(a), Sequence::Range(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::One(a), Sequence::Empty(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::One(a), Sequence::One(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::One(a), Sequence::Many(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::One(a), Sequence::Range(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Many(a), Sequence::Empty(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Many(a), Sequence::One(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Many(a), Sequence::Many(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Many(a), Sequence::Range(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Range(a), Sequence::Empty(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Range(a), Sequence::One(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Range(a), Sequence::Many(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
            (Sequence::Range(a), Sequence::Range(b)) => {
                a.value_compare(b, op, collation, timezone, source)
            }
        }
    }

    pub(crate) fn one_node(&self) -> error::Result<N> {
        match self {
            Sequence::Empty(inner) => inner.one_node(),
            Sequence::One(inner) => inner.one_node(),
            Sequence::Many(inner) => inner.one_node(),
            Sequence::Range(inner) => inner.one_node(),
        }
    }

    pub(crate) fn is(&self, other: &Self) -> error::Result<bool> {
        match (self, other) {
            (Sequence::Empty(a), Sequence::Empty(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Empty(a), Sequence::One(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Empty(a), Sequence::Many(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Empty(a), Sequence::Range(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::One(a), Sequence::Empty(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::One(a), Sequence::One(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::One(a), Sequence::Many(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::One(a), Sequence::Range(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Many(a), Sequence::Empty(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Many(a), Sequence::One(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Many(a), Sequence::Many(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Many(a), Sequence::Range(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Range(a), Sequence::Empty(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Range(a), Sequence::One(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Range(a), Sequence::Many(b)) => SequenceOrder::<_, N>::is(a, b),
            (Sequence::Range(a), Sequence::Range(b)) => SequenceOrder::<_, N>::is(a, b),
        }
    }

    pub(crate) fn precedes<S: XmlDS<Node = N>>(
        &self,
        other: &Self,
        annotations: xml::DocumentOrderAccess<S>,
    ) -> error::Result<bool> {
        match (self, other) {
            (Sequence::Empty(a), Sequence::Empty(b)) => a.precedes(b, annotations),
            (Sequence::Empty(a), Sequence::One(b)) => a.precedes(b, annotations),
            (Sequence::Empty(a), Sequence::Many(b)) => a.precedes(b, annotations),
            (Sequence::Empty(a), Sequence::Range(b)) => a.precedes(b, annotations),
            (Sequence::One(a), Sequence::Empty(b)) => a.precedes(b, annotations),
            (Sequence::One(a), Sequence::One(b)) => a.precedes(b, annotations),
            (Sequence::One(a), Sequence::Many(b)) => a.precedes(b, annotations),
            (Sequence::One(a), Sequence::Range(b)) => a.precedes(b, annotations),
            (Sequence::Many(a), Sequence::Empty(b)) => a.precedes(b, annotations),
            (Sequence::Many(a), Sequence::One(b)) => a.precedes(b, annotations),
            (Sequence::Many(a), Sequence::Many(b)) => a.precedes(b, annotations),
            (Sequence::Many(a), Sequence::Range(b)) => a.precedes(b, annotations),
            (Sequence::Range(a), Sequence::Empty(b)) => a.precedes(b, annotations),
            (Sequence::Range(a), Sequence::One(b)) => a.precedes(b, annotations),
            (Sequence::Range(a), Sequence::Many(b)) => a.precedes(b, annotations),
            (Sequence::Range(a), Sequence::Range(b)) => a.precedes(b, annotations),
        }
    }

    pub(crate) fn follows<S: XmlDS<Node = N>>(
        &self,
        other: &Self,
        annotations: xml::DocumentOrderAccess<S>,
    ) -> error::Result<bool> {
        match (self, other) {
            (Sequence::Empty(a), Sequence::Empty(b)) => a.follows(b, annotations),
            (Sequence::Empty(a), Sequence::One(b)) => a.follows(b, annotations),
            (Sequence::Empty(a), Sequence::Many(b)) => a.follows(b, annotations),
            (Sequence::Empty(a), Sequence::Range(b)) => a.follows(b, annotations),
            (Sequence::One(a), Sequence::Empty(b)) => a.follows(b, annotations),
            (Sequence::One(a), Sequence::One(b)) => a.follows(b, annotations),
            (Sequence::One(a), Sequence::Many(b)) => a.follows(b, annotations),
            (Sequence::One(a), Sequence::Range(b)) => a.follows(b, annotations),
            (Sequence::Many(a), Sequence::Empty(b)) => a.follows(b, annotations),
            (Sequence::Many(a), Sequence::One(b)) => a.follows(b, annotations),
            (Sequence::Many(a), Sequence::Many(b)) => a.follows(b, annotations),
            (Sequence::Many(a), Sequence::Range(b)) => a.follows(b, annotations),
            (Sequence::Range(a), Sequence::Empty(b)) => a.follows(b, annotations),
            (Sequence::Range(a), Sequence::One(b)) => a.follows(b, annotations),
            (Sequence::Range(a), Sequence::Many(b)) => a.follows(b, annotations),
            (Sequence::Range(a), Sequence::Range(b)) => a.follows(b, annotations),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // a static assertion to ensure that Sequence never grows in size
    #[cfg(target_arch = "x86_64")]
    static_assertions::assert_eq_size!(Sequence<xot::Node>, [u8; 24]);
}
