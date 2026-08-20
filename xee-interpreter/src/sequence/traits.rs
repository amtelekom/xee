use xee_datasource::XmlNode;

use crate::function::XmlDS;
use crate::{
    atomic::{self, AtomicCompare},
    context, error, function,
    string::Collation,
    xml,
};

use super::{
    comparison,
    item::Item,
    iter::{self, AtomizedIter, NodeIter},
};

pub(crate) type BoxedItemIter<'a, N> = Box<dyn Iterator<Item = Item<N>> + 'a>; //N should be XmlNode

/// The core sequence interface: a sequence must implement this to function.
///
/// If you do, SequenceExt provides a whole of APIs on top of it.
pub trait SequenceCore<'a, I, N: XmlNode>
where
    I: Iterator<Item = Item<N>>,
{
    /// Check whether the sequence is empty
    fn is_empty(&self) -> bool;

    /// Get the sequence length
    fn len(&self) -> usize;

    /// Get an item in the index, if it exists
    fn get(&self, index: usize) -> Option<Item<N>>;

    /// Get the items from the sequence as an iterator
    fn iter(&'a self) -> I;

    /// Get a single item from the sequence, if it only contains one item
    fn one(self) -> error::Result<Item<N>>;

    /// Get a optional item from the sequence
    fn option(self) -> error::Result<Option<Item<N>>>;

    /// Effective boolean value
    fn effective_boolean_value(&'a self) -> error::Result<bool>;

    /// String value
    fn string_value<S: XmlDS<Node = N>>(&'a self, source: &S) -> error::Result<String>;
}

pub trait SequenceExt<'a, I, N: XmlNode + 'a>: SequenceCore<'a, I, N>
where
    I: Iterator<Item = Item<N>> + 'a,
{
    /// Access an iterator over the nodes in the sequence
    ///
    /// An error is returned for items that are not a node.
    fn nodes(&'a self) -> impl Iterator<Item = error::Result<N>> + 'a {
        NodeIter::new(self.iter())
    }

    /// Access an iterator for the atomized values in the sequence
    fn atomized<S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> impl Iterator<Item = error::Result<atomic::Atomic>> + 'a {
        AtomizedIter::new(source, self.iter())
    }

    /// Get just one atomized value from the sequence
    fn atomized_one<S: XmlDS<Node = N>>(&'a self, source: &'a S) -> error::Result<atomic::Atomic> {
        iter::one(self.atomized(source))?
    }

    /// Get an optional atomized value from the sequence
    fn atomized_option<S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> error::Result<Option<atomic::Atomic>> {
        iter::option(self.atomized(source))?.transpose()
    }

    /// Is used internally by the library macro.
    fn unboxed_atomized<T: 'a, S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
        extract: impl Fn(atomic::Atomic) -> error::Result<T> + 'a,
    ) -> impl Iterator<Item = error::Result<T>> + 'a {
        self.atomized(source).map(move |a| extract(a?))
    }

    /// Access an iterator over the XPath maps in the sequence
    ///
    /// An error is returned for items that are not a map.
    fn map_iter(&'a self) -> impl Iterator<Item = error::Result<function::Map<N>>> {
        self.iter().map(|item| item.to_map())
    }

    /// Access an iterator over the XPath arrays in the sequence
    ///
    /// An error is returned for items that are not an array.
    fn array_iter(&'a self) -> impl Iterator<Item = error::Result<function::Array<N>>> {
        self.iter().map(|item| item.to_array())
    }

    /// Access an iterator over elements nodes in the sequence
    ///
    /// An error is returned for items that are not an element.
    fn elements<S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> error::Result<impl Iterator<Item = error::Result<S::Node>>> {
        Ok(self.nodes().map(|n| match n {
            Ok(n) => {
                if source.is_element(n) {
                    Ok(n)
                } else {
                    Err(error::Error::XPTY0004(
                        "SequenceExt::elements called on a sequence containing a non-element node",
                    ))
                }
            }
            Err(n) => Err(n),
        }))
    }

    /// Create an XPath array from this sequence.
    fn to_array(&'a self) -> error::Result<function::Array<N>> {
        let mut array = Vec::with_capacity(self.len());
        for item in self.iter() {
            array.push(item.into());
        }
        // TODO: array.into() is somehow returning a Result, that seems weird
        // if this is really fallible, it should be try into. If it's not,
        // this whole function should be infallible.
        Ok(array.into())
    }
}

pub(crate) trait SequenceCompare<'a, I, N: XmlNode + 'a>: SequenceExt<'a, I, N>
where
    I: Iterator<Item = Item<N>> + 'a,
{
    fn general_comparison<O, J, S: XmlDS<Node = N>>(
        &'a self,
        other: &'a impl SequenceExt<'a, J, N>,
        op: O,
        context: &context::DynamicContext<S>,
        source: &'a S,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
        J: Iterator<Item = Item<N>> + 'a,
    {
        let a_atomized = self.atomized(source);
        let b_atomized = other.atomized(source);
        // optimization:
        // if a is actually smaller than b, then we want to pass a as the second argument,
        // because a gets collected by the general comparison logic, and we'd rather
        // collect the smallest vector
        let (a_lower_bound, _) = a_atomized.size_hint();
        let (b_lower_bound, _) = b_atomized.size_hint();
        // if the lower bound of a is smaller than that of b, we invert the comparison
        // so that a (a smaller sequence) is actually collected and then compared with.
        if a_lower_bound < b_lower_bound {
            comparison::general_comparison(b_atomized, a_atomized, context, O::arguments_inverted())
        } else {
            comparison::general_comparison(a_atomized, b_atomized, context, op)
        }
    }

    fn value_compare<O, J, S: XmlDS<Node = N>>(
        &'a self,
        other: &'a impl SequenceExt<'a, J, N>,
        _op: O,
        collation: &Collation,
        timezone: chrono::FixedOffset,
        source: &'a S,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
        J: Iterator<Item = Item<N>> + 'a,
    {
        let a = self.atomized_one(source)?;
        let b = other.atomized_one(source)?;
        O::atomic_compare(a, b, |a: &str, b: &str| collation.compare(a, b), timezone)
    }
}

pub(crate) trait SequenceOrder<'a, I, N: XmlNode>: SequenceCore<'a, I, N>
where
    I: Iterator<Item = Item<N>>,
{
    fn one_node(&self) -> error::Result<N>;

    fn is<J>(&'a self, other: &'a impl SequenceOrder<'a, J, N>) -> error::Result<bool>
    where
        J: Iterator<Item = Item<N>> + 'a,
    {
        let a = self.one_node()?;
        let b = other.one_node()?;
        Ok(a == b)
    }

    fn precedes<J, S: XmlDS<Node = N>>(
        &'a self,
        other: &'a impl SequenceOrder<'a, J, N>,
        annotations: xml::DocumentOrderAccess<S>,
    ) -> error::Result<bool>
    where
        J: Iterator<Item = Item<N>> + 'a,
    {
        let a = self.one_node()?;
        let b = other.one_node()?;
        let a_order = annotations.get(a);
        let b_order = annotations.get(b);
        Ok(a_order < b_order)
    }

    fn follows<J, S: XmlDS<Node = N>>(
        &'a self,
        other: &'a impl SequenceOrder<'a, J, N>,
        annotations: xml::DocumentOrderAccess<S>,
    ) -> error::Result<bool>
    where
        J: Iterator<Item = Item<N>> + 'a,
    {
        let a = self.one_node()?;
        let b = other.one_node()?;
        let a_order = annotations.get(a);
        let b_order = annotations.get(b);
        Ok(a_order > b_order)
    }
}
