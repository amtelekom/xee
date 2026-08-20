use std::marker::PhantomData;

use ibig::IBig;
use xee_datasource::{XmlDS, XmlNode};

use crate::atomic::AtomicCompareValue;
use crate::error;
use crate::sequence::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use crate::sequence::Item;

// this size should be below a usize
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    start: Box<IBig>,
    end: Box<IBig>,
}

impl Range {
    pub(crate) fn new(start: IBig, end: IBig) -> error::Result<Self> {
        let length: IBig = &end - &start;
        if length > MAXIMUM_RANGE_SIZE.into() {
            return Err(error::Error::FOAR0002);
        }

        Ok(Range {
            start: start.into(),
            end: end.into(),
        })
    }

    pub(crate) fn start(&self) -> &IBig {
        &self.start
    }
    pub(crate) fn end(&self) -> &IBig {
        &self.end
    }

    pub(crate) fn contains(&self, index: &IBig) -> bool {
        index >= self.start.as_ref() && index < self.end.as_ref()
    }

    pub(crate) fn general_comparison_integer(
        &self,
        value: &IBig,
        comparison: AtomicCompareValue,
    ) -> bool {
        match comparison {
            // value has to be within the range
            AtomicCompareValue::Eq => value >= self.start.as_ref() && value < self.end.as_ref(),
            // value has to be outside the range
            AtomicCompareValue::Ne => value < self.start.as_ref() || value >= self.end.as_ref(),
            // value has to be greater than the start
            // 10 gt 10..11 is false
            AtomicCompareValue::Gt => value > self.start.as_ref(),
            // value has to be less than the end - 1
            // 10 lt 10..11 is false
            AtomicCompareValue::Lt => {
                let one: IBig = 1.into();
                let end = self.end.as_ref() - &one;
                value < &end
            }
            // value has to be greater than or equal to the start
            // 10 ge 10..11 is true
            AtomicCompareValue::Ge => value >= self.start.as_ref(),
            // value has to be less than the end
            // 10 le 10..11 is true
            AtomicCompareValue::Le => value < self.end.as_ref(),
        }
    }
}

impl<'a, N: XmlNode> SequenceCore<'a, RangeIterator<N>, N> for Range {
    #[inline]
    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    fn len(&self) -> usize {
        let len = self.end.as_ref() - self.start.as_ref();
        // We should prevent any range that's > usize from being crated
        len.try_into().unwrap()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item<N>> {
        if index < SequenceCore::<'a, RangeIterator<N>, N>::len(self) {
            let i: IBig = self.start.as_ref() + index;
            Some(i.into())
        } else {
            None
        }
    }

    #[inline]
    fn one(self) -> error::Result<Item<N>> {
        match SequenceCore::<'a, RangeIterator<N>, N>::len(&self) {
            0 => Err(error::Error::XPTY0004(
                "Range::one called on an empty range",
            )),
            1 => {
                let i: IBig = self.start.as_ref().clone();
                Ok(i.into())
            }
            _ => Err(error::Error::XPTY0004(
                "Range::one called on a range with more than one item",
            )),
        }
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item<N>>> {
        match SequenceCore::<'a, RangeIterator<N>, N>::len(&self) {
            0 => Ok(None),
            1 => {
                let i: IBig = self.start.as_ref().clone();
                Ok(Some(i.into()))
            }
            _ => Err(error::Error::XPTY0004(
                "Range::option called on a range with more than one item",
            )),
        }
    }

    #[inline]
    fn iter(&'a self) -> RangeIterator<N> {
        RangeIterator {
            start: self.start.as_ref().clone(),
            end: self.end.as_ref().clone(),
            index: 0.into(),
            _phantom: PhantomData {},
        }
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        match SequenceCore::<'a, RangeIterator<N>, N>::len(self) {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(error::Error::FORG0006),
        }
    }

    #[inline]
    fn string_value<S: XmlDS<Node = N>>(&self, _source: &S) -> error::Result<String> {
        match SequenceCore::<'a, RangeIterator<N>, N>::len(self) {
            0 => Ok(String::new()),
            1 => Ok(self.start.to_string()),
            _ => Err(error::Error::XPTY0004(
                "Range::string_value called on a range with more than one item",
            )),
        }
    }
}

pub struct RangeIterator<N: XmlNode> {
    start: IBig,
    end: IBig,
    index: IBig,
    _phantom: PhantomData<N>,
}

impl<N: XmlNode> RangeIterator<N> {
    pub(crate) fn new(start: IBig, end: IBig) -> Self {
        Self {
            start,
            end,
            index: 0.into(),
            _phantom: PhantomData {},
        }
    }
}

impl<N: XmlNode> Iterator for RangeIterator<N> {
    type Item = Item<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < (&self.end - &self.start) {
            let i = &self.start + &self.index;
            self.index += 1;
            Some(i.into())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = &self.end - &self.start;
        // we know that we don't have a range that's > usize as we cannot construct
        // any
        let len: usize = len.try_into().expect("range size is within usize");
        (len, Some(len))
    }
}

impl<'a, I, N: XmlNode + 'a> SequenceExt<'a, I, N> for Range
where
    I: Iterator<Item = Item<N>> + 'a,
    Range: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode + 'a> SequenceCompare<'a, I, N> for Range
where
    I: Iterator<Item = Item<N>> + 'a,
    Range: SequenceCore<'a, I, N>,
{
}

impl<'a, I, N: XmlNode + 'a> SequenceOrder<'a, I, N> for Range
where
    I: Iterator<Item = Item<N>>,
    Range: SequenceCore<'a, I, N>,
{
    fn one_node(&self) -> error::Result<N> {
        // a range never contains nodes
        Err(error::Error::XPTY0004(
            "Range::one_node called on a Range, which never contains nodes",
        ))
    }
}
