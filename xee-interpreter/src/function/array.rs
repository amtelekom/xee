use std::rc::Rc;

use xee_datasource::XmlNode;

use super::XmlDS;
use crate::{context, error, sequence, string};

/// An XPath Array
///
/// Not to be confused with an XPath sequence, this is a type of item that can exist
/// in a sequence when you need to have an actual list.
///
/// I tried to make this a Rc<[]> but this is bigger and blows up the item as a result.
#[derive(Debug, Clone, PartialEq)]
pub struct Array<N: XmlNode>(pub(crate) Rc<Vec<sequence::Sequence<N>>>);

impl<N: XmlNode> Array<N> {
    pub(crate) fn new(vec: Vec<sequence::Sequence<N>>) -> Self {
        Self(vec.into())
    }

    pub(crate) fn join(arrays: &[Self]) -> Self {
        let mut vec = Vec::with_capacity(arrays.iter().map(|array| array.0.len()).sum());
        for array in arrays {
            vec.extend(array.0.as_ref().iter().cloned());
        }
        Self::new(vec)
    }

    pub(crate) fn index(&self, index: usize) -> Option<&sequence::Sequence<N>> {
        self.0.get(index)
    }

    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &sequence::Sequence<N>> {
        self.0.iter()
    }

    pub(crate) fn put(&self, index: usize, member: &sequence::Sequence<N>) -> Option<Self> {
        if index >= self.0.len() {
            return None;
        }
        let mut vec = self.0.as_ref().to_vec();
        vec[index] = member.clone();
        Some(Self::new(vec))
    }

    pub(crate) fn append(&self, appendage: &sequence::Sequence<N>) -> Self {
        let mut vec = self.0.as_ref().to_vec();
        vec.push(appendage.clone());
        Self::new(vec)
    }

    pub(crate) fn subarray(&self, start: usize, length: usize) -> Option<Self> {
        if start > self.0.len() || (start + length) > self.0.len() {
            return None;
        }
        let mut vec = Vec::with_capacity(length);
        for i in start..(start + length) {
            vec.push(self.0[i].clone());
        }
        Some(Self::new(vec))
    }

    pub(crate) fn remove_positions(&self, positions: &[usize]) -> Option<Self> {
        for position in positions {
            if position >= &self.0.len() {
                return None;
            }
        }
        let mut vec = Vec::with_capacity(self.0.len() - positions.len());

        for (i, member) in self.0.iter().enumerate() {
            if !positions.contains(&i) {
                vec.push(member.clone());
            }
        }
        Some(Self::new(vec))
    }

    pub(crate) fn reversed(&self) -> Self {
        let mut vec = self.0.as_ref().to_vec();
        vec.reverse();
        Self::new(vec)
    }

    pub(crate) fn insert_before(
        &self,
        position: usize,
        member: &sequence::Sequence<N>,
    ) -> Option<Self> {
        if position > self.0.len() {
            return None;
        }
        let mut vec = self.0.as_ref().to_vec();
        vec.insert(position, member.clone());
        Some(Self::new(vec))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn deep_equal<S: XmlDS<Node = N>>(
        &self,
        other: Array<N>,
        collation: &string::Collation,
        default_offset: chrono::FixedOffset,
        source: &S,
    ) -> error::Result<bool> {
        if self.0.len() != other.0.len() {
            return Ok(false);
        }
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if !a.deep_equal(b, collation, default_offset, source)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn display_representation<S: XmlDS<Node = N>>(
        &self,
        source: &S,
        context: &context::DynamicContext<S>,
    ) -> String {
        let members = self
            .0
            .iter()
            .map(|member| member.display_representation(source, context))
            .collect::<Vec<_>>();
        format!("[\n{}\n]", members.join(",\n"))
    }
}

impl<N: XmlNode> From<Vec<sequence::Sequence<N>>> for Array<N> {
    fn from(vec: Vec<sequence::Sequence<N>>) -> Self {
        Self::new(vec)
    }
}

#[cfg(test)]
mod tests {

    #[cfg(target_arch = "x86_64")]
    static_assertions::assert_eq_size!(super::Array<xot::Node>, [u8; 8]);
}
