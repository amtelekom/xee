use std::rc::Rc;

use xee_datasource::{XmlDS, XmlNode};

use crate::error;
use crate::sequence::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use crate::sequence::Item;

// this size should be below a usize
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone, PartialEq)]
pub struct Many<N: XmlNode> {
    items: Rc<[Item<N>]>,
}

impl<N: XmlNode> Many<N> {}

impl<N: XmlNode> From<Vec<Item<N>>> for Many<N> {
    fn from(items: Vec<Item<N>>) -> Self {
        Many::<N> {
            items: items.into(),
        }
    }
}

impl<'a, N: XmlNode> SequenceCore<'a, std::iter::Cloned<std::slice::Iter<'a, Item<N>>>, N>
    for Many<N>
{
    #[inline]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item<N>> {
        self.items.get(index).cloned()
    }

    #[inline]
    fn one(self) -> error::Result<Item<N>> {
        Err(error::Error::XPTY0004(
            "Many::one called on a sequence with more than one item",
        ))
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item<N>>> {
        Err(error::Error::XPTY0004(
            "Many::option called on a sequence with more than one item",
        ))
    }

    #[inline]
    fn iter(&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Item<N>>> {
        self.items.iter().cloned()
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        // handle the case where the first item is a node
        // it has to be a singleton otherwise
        if matches!(self.items[0], Item::Node(_)) {
            Ok(true)
        } else {
            Err(error::Error::FORG0006)
        }
    }

    #[inline]
    fn string_value<S: XmlDS<Node = N>>(&self, _source: &S) -> error::Result<String> {
        Err(error::Error::XPTY0004(
            "Many::string_value called on a sequence with more than one item",
        ))
    }
}

impl<'a, I, N: XmlNode + 'a> SequenceExt<'a, I, N> for Many<N>
where
    I: Iterator<Item = Item<N>> + 'a,
    Many<N>: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode + 'a> SequenceCompare<'a, I, N> for Many<N>
where
    I: Iterator<Item = Item<N>> + 'a,
    Many<N>: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode + 'a> SequenceOrder<'a, I, N> for Many<N>
where
    I: Iterator<Item = Item<N>>,
    Many<N>: SequenceCore<'a, I, N>,
{
    fn one_node(&self) -> error::Result<N> {
        Err(error::Error::XPTY0004(
            "Many::one_node called on a sequence with more than one item",
        ))
    }
}
