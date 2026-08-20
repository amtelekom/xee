use xee_datasource::{XmlNode, XmlNodeWrapper};

use crate::{atomic, error, function};

use super::{Item, Sequence};

// turn a single item into a sequence
impl<N: XmlNode> From<Item<N>> for Sequence<N> {
    fn from(item: Item<N>) -> Self {
        Sequence::One(item.into())
    }
}

// turn a single node into a sequence
impl<N: XmlNode> From<XmlNodeWrapper<N>> for Sequence<N> {
    fn from(node: XmlNodeWrapper<N>) -> Self {
        let item: Item<N> = node.into();
        item.into()
    }
}

// turn a sequence into a single node
// impl<N: XmlNode> TryFrom<Sequence<N>> for N {
//     type Error = error::Error;

//     fn try_from(sequence: Sequence<N>) -> Result<Self, Self::Error> {
//         match sequence {
//             Sequence::One(item) => item.item().try_into(),
//             _ => Err(error::Error::XPTY0004),
//         }
//     }
// }

impl<N: XmlNode> TryInto<XmlNodeWrapper<N>> for Sequence<N> {
    type Error = error::Error;

    fn try_into(self) -> Result<XmlNodeWrapper<N>, Self::Error> {
        match self {
            Sequence::One(item) => Ok(item.item().try_into()?),
            _ => Err(error::Error::XPTY0004(
                "XmlNodeWrapper::try_from called with a Sequence that is not a single item",
            )),
        }
    }
}

impl<N: XmlNode> TryFrom<Sequence<N>> for function::Function<N> {
    type Error = error::Error;

    fn try_from(sequence: Sequence<N>) -> Result<Self, Self::Error> {
        match sequence {
            Sequence::One(item) => item.item().try_into(),
            _ => Err(error::Error::XPTY0004(
                "Function::try_from called with a Sequence that is not a single item",
            )),
        }
    }
}

// turn a single array into a sequence
impl<N: XmlNode> From<function::Array<N>> for Sequence<N> {
    fn from(array: function::Array<N>) -> Self {
        let item: Item<N> = array.into();
        item.into()
    }
}

// turn a sequence into an array
impl<N: XmlNode> From<Sequence<N>> for function::Array<N> {
    fn from(sequence: Sequence<N>) -> Self {
        let items = sequence
            .iter()
            .map(|item| {
                let sequence: Sequence<N> = item.into();
                sequence
            })
            .collect::<Vec<_>>();

        Self::new(items)
    }
}

// turn a single map into a sequence
impl<N: XmlNode> From<function::Map<N>> for Sequence<N> {
    fn from(map: function::Map<N>) -> Self {
        let item: Item<N> = map.into();
        item.into()
    }
}

// turn an option that can be turned into an item into a sequence
impl<T, N: XmlNode> From<Option<T>> for Sequence<N>
where
    T: Into<Item<N>>,
{
    fn from(item: Option<T>) -> Self {
        match item {
            Some(item) => {
                let item = item.into();
                Sequence::One(item.into())
            }
            None => Sequence::default(),
        }
    }
}

// turn something that can be turned into an atomic into a sequence
impl<T, N: XmlNode> From<T> for Sequence<N>
where
    T: Into<atomic::Atomic>,
{
    fn from(atomic: T) -> Self {
        let atomic: atomic::Atomic = atomic.into();
        let item: Item<N> = atomic.into();
        Sequence::One(item.into())
    }
}

impl<T, N: XmlNode> From<Vec<T>> for Sequence<N>
where
    T: Into<Item<N>>,
{
    fn from(values: Vec<T>) -> Self {
        let mut items = Vec::with_capacity(values.len());
        for value in values {
            let item: Item<N> = value.into();
            items.push(item);
        }
        Sequence::new(items)
    }
}

// turn an iterator of things that can be turned into items into a sequence
impl<N: XmlNode> FromIterator<Item<N>> for Sequence<N> {
    fn from_iter<I: IntoIterator<Item = Item<N>>>(iter: I) -> Self {
        let items = iter.into_iter().collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of item references into a sequence
impl<'a, N: XmlNode> FromIterator<&'a Item<N>> for Sequence<N> {
    fn from_iter<I: IntoIterator<Item = &'a Item<N>>>(iter: I) -> Self {
        let items = iter.into_iter().cloned().collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of atomics into a sequence
impl<N: XmlNode> FromIterator<atomic::Atomic> for Sequence<N> {
    fn from_iter<I: IntoIterator<Item = atomic::Atomic>>(iter: I) -> Self {
        let items = iter.into_iter().map(Item::from).collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of nodes into a sequence
impl<N: XmlNode> FromIterator<N> for Sequence<N> {
    fn from_iter<I: IntoIterator<Item = N>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(XmlNodeWrapper)
            .map(Item::from)
            .collect::<Vec<_>>();
        items.into()
    }
}
