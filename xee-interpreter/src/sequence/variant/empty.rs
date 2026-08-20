use xee_datasource::{XmlDS, XmlNode};

use crate::{atomic, error};

use crate::sequence::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use crate::sequence::Item;

// this size should be below a usize
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl<N: XmlNode> SequenceCore<'_, std::iter::Empty<Item<N>>, N> for Empty {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn get(&self, _index: usize) -> Option<Item<N>> {
        None
    }

    #[inline]
    fn one(self) -> error::Result<Item<N>> {
        Err(error::Error::XPTY0004(
            "Empty::one called on an empty sequence",
        ))
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item<N>>> {
        Ok(None)
    }

    #[inline]
    fn iter(&self) -> std::iter::Empty<Item<N>> {
        std::iter::empty()
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        Ok(false)
    }

    #[inline]
    fn string_value<S: XmlDS<Node = N>>(&self, _source: &S) -> error::Result<String> {
        Ok(String::new())
    }
}

// We implement specific interfaces here, instead of generically for all
// T that implement sequenceCore, because we want to provide a few specialized
// implementations.
impl<'a, I, N: XmlNode + 'a> SequenceExt<'a, I, N> for Empty
where
    I: Iterator<Item = Item<N>> + 'a,
    Empty: SequenceCore<'a, I, N>,
{
    fn atomized<S: XmlDS<Node = N>>(
        &'a self,
        _source: &'a S,
    ) -> impl Iterator<Item = error::Result<atomic::Atomic>> + 'a {
        std::iter::empty()
    }

    /// Get just one atomized value from the sequence
    fn atomized_one<S: XmlDS<Node = N>>(&'a self, _source: &'a S) -> error::Result<atomic::Atomic> {
        Err(error::Error::XPTY0004(
            "Empty::atomized_one called on an empty sequence",
        ))
    }

    /// Get an optional atomized value from the sequence
    fn atomized_option<S: XmlDS<Node = N>>(
        &'a self,
        _source: &'a S,
    ) -> error::Result<Option<atomic::Atomic>> {
        Ok(None)
    }
}

impl<'a, I, N: XmlNode> SequenceCompare<'a, I, N> for Empty
where
    I: Iterator<Item = Item<N>> + 'a,
    Empty: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode> SequenceOrder<'a, I, N> for Empty
where
    I: Iterator<Item = Item<N>>,
    Empty: SequenceCore<'a, I, N>,
{
    fn one_node(&self) -> error::Result<N> {
        Err(error::Error::XPTY0004(
            "Empty::one_node called on an empty sequence",
        ))
    }
}
