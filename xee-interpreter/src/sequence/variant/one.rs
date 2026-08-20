use xee_datasource::XmlNode;

use crate::function::XmlDS;
use crate::sequence::item::Item;
use crate::sequence::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use crate::sequence::AtomizedItemIter;
use crate::{atomic, error};

// this size should be below a usize
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone, PartialEq)]
pub struct One<N: XmlNode> {
    item: Item<N>,
}

impl<N: XmlNode> One<N> {
    pub(crate) fn item(&self) -> &Item<N> {
        &self.item
    }

    pub(crate) fn into_item(self) -> Item<N> {
        self.item
    }
}

impl<N: XmlNode> From<Item<N>> for One<N> {
    fn from(item: Item<N>) -> Self {
        One { item }
    }
}

impl<N: XmlNode> From<One<N>> for Item<N> {
    fn from(one: One<N>) -> Self {
        one.item
    }
}

impl<'a, N: XmlNode> SequenceCore<'a, std::iter::Once<Item<N>>, N> for One<N> {
    #[inline]
    fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item<N>> {
        if index == 0 {
            Some(self.item.clone())
        } else {
            None
        }
    }

    #[inline]
    fn one(self) -> error::Result<Item<N>> {
        Ok(self.item)
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item<N>>> {
        Ok(Some(self.item))
    }

    #[inline]
    fn iter(&'a self) -> std::iter::Once<Item<N>> {
        std::iter::once(self.item.clone())
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        self.item.effective_boolean_value()
    }

    #[inline]
    fn string_value<S: XmlDS<Node = N>>(&self, source: &S) -> error::Result<String> {
        self.item.string_value(source)
    }
}

impl<'a, I, N: XmlNode + 'a> SequenceExt<'a, I, N> for One<N>
where
    I: Iterator<Item = Item<N>> + 'a,
    One<N>: SequenceCore<'a, I, N>,
{
    fn atomized<S: XmlDS<Node = N>>(
        &'a self,
        source: &'a S,
    ) -> impl Iterator<Item = error::Result<atomic::Atomic>> + 'a {
        AtomizedItemIter::new(&self.item, source)
    }
}

impl<'a, I, N: XmlNode> SequenceCompare<'a, I, N> for One<N>
where
    I: Iterator<Item = Item<N>> + 'a,
    One<N>: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode> SequenceOrder<'a, I, N> for One<N>
where
    I: Iterator<Item = Item<N>>,
    One<N>: SequenceCore<'a, I, N>,
{
    fn one_node(&self) -> error::Result<N> {
        match &self.item {
            Item::Node(n) => Ok(*n),
            _ => Err(error::Error::XPTY0004(
                "One::one_node called on a non-Node Item",
            )),
        }
    }
}
