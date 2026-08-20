use std::rc::Rc;

use crate::function::XmlDS;
use ahash::{HashMap, HashMapExt};
use xee_datasource::XmlNode;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::{atomic, context, error, sequence, string};

/// An XPath Map (a collection of key-value pairs).
#[derive(Debug, Clone, PartialEq)]
pub enum Map<N: XmlNode> {
    Empty(EmptyMap),
    One(OneMap<N>),
    Many(ManyMap<N>),
}

#[allow(type_alias_bounds)]
pub type Entry<N: XmlNode> = (atomic::Atomic, sequence::Sequence<N>);

impl<N: XmlNode> Map<N> {
    pub(crate) fn new(entries: Vec<Entry<N>>) -> error::Result<Self> {
        match entries.len() {
            0 => Ok(Self::Empty(EmptyMap)),
            1 => {
                let (key, value) = entries.into_iter().next().unwrap();
                let map_key = atomic::MapKey::new(key.clone())?;
                Ok(Self::One(
                    OneMapValue {
                        map_key,
                        key_value: (key, value),
                    }
                    .into(),
                ))
            }
            _ => Ok(Self::Many(ManyMap::new(entries)?)),
        }
    }

    fn from_map(map: HashMap<atomic::MapKey, Entry<N>>) -> Self {
        match map.len() {
            0 => Self::Empty(EmptyMap),
            1 => {
                let (map_key, (key, value)) = map.into_iter().next().unwrap();
                Self::One(
                    OneMapValue {
                        map_key,
                        key_value: (key, value),
                    }
                    .into(),
                )
            }
            _ => Self::Many(ManyMap(Rc::new(map))),
        }
    }

    pub(crate) fn combine(
        maps: impl Iterator<Item = error::Result<Map<N>>>,
        combine: impl Fn(
            sequence::Sequence<N>,
            sequence::Sequence<N>,
        ) -> error::Result<sequence::Sequence<N>>,
    ) -> error::Result<Map<N>> {
        let mut result = HashMap::new();
        for map in maps {
            for (map_key, (key, value)) in map?.full_entries() {
                let map_key = map_key.clone();
                let entry = result.remove(&map_key);
                let value = if let Some((_, a)) = entry {
                    combine(a, value.clone())?
                } else {
                    value.clone()
                };
                result.insert(map_key, (key.clone(), value));
            }
        }
        Ok(Map::from_map(result))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Map::Empty(map) => Mappable::<N>::len(map),
            Map::One(map) => map.len(),
            Map::Many(map) => map.len(),
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Map::Empty(map) => Mappable::<N>::is_empty(map),
            Map::One(map) => map.is_empty(),
            Map::Many(map) => map.is_empty(),
        }
    }
    pub(crate) fn get(&self, key: &atomic::Atomic) -> Option<&sequence::Sequence<N>> {
        match self {
            Map::Empty(map) => map.get(key),
            Map::One(map) => map.get(key),
            Map::Many(map) => map.get(key),
        }
    }
    pub(crate) fn keys(&self) -> Box<dyn Iterator<Item = &atomic::Atomic> + '_> {
        match self {
            Map::Empty(map) => Box::new(Mappable::<N>::keys(map)),
            Map::One(map) => Box::new(map.keys()),
            Map::Many(map) => Box::new(map.keys()),
        }
    }
    pub(crate) fn entries(
        &self,
    ) -> Box<dyn Iterator<Item = (&atomic::Atomic, &sequence::Sequence<N>)> + '_> {
        match self {
            Map::Empty(map) => Box::new(map.entries()),
            Map::One(map) => Box::new(map.entries()),
            Map::Many(map) => Box::new(map.entries()),
        }
    }

    pub(crate) fn map_keys(&self) -> Box<dyn Iterator<Item = &'_ atomic::MapKey> + '_> {
        match self {
            Map::Empty(map) => Box::new(Mappable::<N>::map_keys(map)),
            Map::One(map) => Box::new(map.map_keys()),
            Map::Many(map) => Box::new(map.map_keys()),
        }
    }

    pub(crate) fn map_key_entries(
        &self,
    ) -> Box<dyn Iterator<Item = (&atomic::MapKey, &sequence::Sequence<N>)> + '_> {
        match self {
            Map::Empty(map) => Box::new(map.map_key_entries()),
            Map::One(map) => Box::new(map.map_key_entries()),
            Map::Many(map) => Box::new(map.map_key_entries()),
        }
    }

    pub(crate) fn full_entries(
        &self,
    ) -> Box<dyn Iterator<Item = (&atomic::MapKey, &Entry<N>)> + '_> {
        match self {
            Map::Empty(map) => Box::new(map.full_entries()),
            Map::One(map) => Box::new(map.full_entries()),
            Map::Many(map) => Box::new(map.full_entries()),
        }
    }

    pub(crate) fn get_as_type<S: XmlDS<Node = N>>(
        &self,
        key: &atomic::Atomic,
        occurrence: ast::Occurrence,
        atomic_type: Xs,
        static_context: &context::StaticContext<S>,
        source: &S,
    ) -> error::Result<Option<sequence::Sequence<N>>> {
        match self {
            Map::Empty(map) => {
                map.get_as_type(key, occurrence, atomic_type, static_context, source)
            }
            Map::One(map) => map.get_as_type(key, occurrence, atomic_type, static_context, source),
            Map::Many(map) => map.get_as_type(key, occurrence, atomic_type, static_context, source),
        }
    }

    pub(crate) fn deep_equal<S: XmlDS<Node = N>>(
        &self,
        other: &Map<N>,
        collation: &string::Collation,
        default_offset: chrono::FixedOffset,
        source: &S,
    ) -> error::Result<bool> {
        match (self, other) {
            (Map::Empty(_), Map::Empty(_)) => Ok(true),
            (Map::Empty(_), _) => Ok(false),
            (_, Map::Empty(_)) => Ok(false),
            (Map::One(map), Map::One(other)) => {
                map.deep_equal(other, collation, default_offset, source)
            }
            (Map::One(map), Map::Many(other)) => {
                map.deep_equal(other, collation, default_offset, source)
            }
            (Map::Many(map), Map::Many(other)) => {
                map.deep_equal(other, collation, default_offset, source)
            }
            (Map::Many(map), Map::One(other)) => {
                map.deep_equal(other, collation, default_offset, source)
            }
        }
    }

    pub fn display_representation<S: XmlDS<Node = N>>(
        &self,
        source: &S,
        context: &context::DynamicContext<S>,
    ) -> String {
        match self {
            Map::Empty(map) => map.display_representation(source, context),
            Map::One(map) => map.display_representation(source, context),
            Map::Many(map) => map.display_representation(source, context),
        }
    }

    pub(crate) fn put(
        &self,
        key: atomic::Atomic,
        value: &sequence::Sequence<N>,
    ) -> error::Result<Self> {
        Ok(match self {
            Map::Empty(_) => {
                // if we add a key to an empty map we get a OneMap
                let map_key = atomic::MapKey::new(key.clone())?;
                Map::One(
                    OneMapValue {
                        map_key,
                        key_value: (key, value.clone()),
                    }
                    .into(),
                )
            }
            Map::One(one) => {
                let map_key = atomic::MapKey::new(key.clone())?;
                if one.0.map_key == map_key {
                    // we merely update the value
                    Map::One(
                        OneMapValue {
                            map_key,
                            key_value: (key.clone(), value.clone()),
                        }
                        .into(),
                    )
                } else {
                    // if we add a key to a one map we get a ManyMap
                    let entries = vec![
                        (one.0.key_value.0.clone(), one.0.key_value.1.clone()),
                        (key, value.clone()),
                    ];
                    Map::Many(ManyMap::try_from(entries)?)
                }
            }
            // since at most we add keys, this cannot turn into a non-many map
            Map::Many(map) => Map::Many(map.put(key, value)?),
        })
    }

    pub(crate) fn remove_keys(
        &self,
        keys: impl Iterator<Item = error::Result<atomic::Atomic>>,
    ) -> error::Result<Self> {
        Ok(match self {
            Map::Empty(_) => Map::Empty(EmptyMap),
            Map::One(map) => {
                for key in keys {
                    let map_key = atomic::MapKey::new(key?.clone())?;
                    if map.0.map_key == map_key {
                        return Ok(Map::Empty(EmptyMap));
                    }
                }
                Map::One(map.clone())
            }
            Map::Many(map) => Map::from_map(map.remove_keys(keys)?),
        })
    }
}

pub(crate) trait Mappable<N: XmlNode> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    // get a key by underlying map key
    fn get_by_map_key(&self, map_key: &atomic::MapKey) -> Option<&sequence::Sequence<N>>;

    /// get map keys
    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_;

    // get map entries, key is map key
    fn map_key_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence<N>)> + '_;

    fn full_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &Entry<N>)> + '_;

    // get a key by atomic
    fn get(&self, key: &atomic::Atomic) -> Option<&sequence::Sequence<N>> {
        let map_key = atomic::MapKey::new(key.clone()).ok()?;
        self.get_by_map_key(&map_key)
    }

    // get atomic keys
    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_;

    // get entries with atomic key and value
    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence<N>)> + '_;

    // get a key by atomic and convert to a specific type
    fn get_as_type<S: XmlDS<Node = N>>(
        &self,
        key: &atomic::Atomic,
        occurrence: ast::Occurrence,
        atomic_type: Xs,
        static_context: &context::StaticContext<S>,
        source: &S,
    ) -> error::Result<Option<sequence::Sequence<N>>> {
        let value = self.get(key);
        let value = match value {
            Some(value) => value,
            None => return Ok(None),
        };
        let sequence_type = ast::SequenceType::Item(ast::Item {
            occurrence,
            item_type: ast::ItemType::AtomicOrUnionType(atomic_type),
        });
        // TODO: is value clone really needed?
        Ok(Some(
            value.clone().sequence_type_matching_function_conversion(
                &sequence_type,
                static_context,
                source,
                // typed function tests can't be invoked
                &|_function| unreachable!(),
            )?,
        ))
    }

    // deep equal compare between two mappables
    fn deep_equal<S: XmlDS<Node = N>>(
        &self,
        other: &impl Mappable<N>,
        collation: &string::Collation,
        default_offset: chrono::FixedOffset,
        source: &S,
    ) -> error::Result<bool> {
        if self.len() != other.len() {
            return Ok(false);
        }
        for (map_key, value) in self.map_key_entries() {
            let other_value = other.get_by_map_key(map_key);
            if let Some(other_value) = other_value {
                if !value.deep_equal(other_value, collation, default_offset, source)? {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn display_representation<S: XmlDS<Node = N>>(
        &self,
        source: &S,
        context: &context::DynamicContext<S>,
    ) -> String {
        let mut entries = self
            .entries()
            .map(|(key, value)| {
                format!(
                    "{}: {}",
                    key.xpath_representation(),
                    value.display_representation(source, context)
                )
            })
            .collect::<Vec<_>>();
        entries.sort();
        format!("map {{\n{}\n}}", entries.join(",\n"))
    }
}

// empty map
#[derive(Debug, Clone, PartialEq)]
pub struct EmptyMap;

impl<N: XmlNode> Mappable<N> for EmptyMap {
    fn len(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn get_by_map_key(&self, _map_key: &atomic::MapKey) -> Option<&sequence::Sequence<N>> {
        None
    }

    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_ {
        std::iter::empty()
    }

    fn map_key_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence<N>)> + '_ {
        std::iter::empty()
    }

    fn full_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &Entry<N>)> + '_ {
        std::iter::empty()
    }

    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_ {
        std::iter::empty()
    }

    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence<N>)> + '_ {
        std::iter::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OneMap<N: XmlNode>(Box<OneMapValue<N>>);

#[derive(Debug, Clone, PartialEq)]
struct OneMapValue<N: XmlNode> {
    map_key: atomic::MapKey,
    key_value: Entry<N>,
}

impl<N: XmlNode> From<OneMapValue<N>> for OneMap<N> {
    fn from(value: OneMapValue<N>) -> Self {
        Self(Box::new(value))
    }
}

impl<N: XmlNode> Mappable<N> for OneMap<N> {
    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn get_by_map_key(&self, map_key: &atomic::MapKey) -> Option<&sequence::Sequence<N>> {
        if &self.0.map_key == map_key {
            Some(&self.0.key_value.1)
        } else {
            None
        }
    }

    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_ {
        std::iter::once(&self.0.map_key)
    }

    fn map_key_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence<N>)> + '_ {
        std::iter::once((&self.0.map_key, &self.0.key_value.1))
    }

    fn full_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &Entry<N>)> + '_ {
        std::iter::once((&self.0.map_key, &self.0.key_value))
    }

    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_ {
        std::iter::once(&self.0.key_value.0)
    }

    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence<N>)> + '_ {
        std::iter::once((&self.0.key_value.0, &self.0.key_value.1))
    }
}

// a normal map uses a hashmap to store > 1 key-value pairs
#[derive(Debug, Clone, PartialEq)]
pub struct ManyMap<N: XmlNode>(Rc<HashMap<atomic::MapKey, Entry<N>>>);

impl<N: XmlNode> ManyMap<N> {
    fn new(entries: Vec<Entry<N>>) -> error::Result<Self> {
        let mut map = HashMap::new();
        for (key, value) in entries {
            let map_key = atomic::MapKey::new(key.clone())?;
            if map.contains_key(&map_key) {
                return Err(error::Error::XQDY0137);
            }
            map.insert(map_key, (key, value));
        }
        Ok(Self(Rc::new(map)))
    }

    pub(crate) fn put(
        &self,
        key: atomic::Atomic,
        value: &sequence::Sequence<N>,
    ) -> error::Result<Self> {
        let mut map = self.0.as_ref().clone();
        let map_key = atomic::MapKey::new(key.clone())?;
        map.insert(map_key, (key, value.clone()));
        Ok(Self(Rc::new(map)))
    }

    pub(crate) fn remove_keys(
        &self,
        keys: impl Iterator<Item = error::Result<atomic::Atomic>>,
    ) -> error::Result<HashMap<atomic::MapKey, Entry<N>>> {
        let mut map = self.0.as_ref().clone();
        for key in keys {
            let map_key = atomic::MapKey::new(key?.clone()).unwrap();
            map.remove(&map_key);
        }
        Ok(map)
    }
}

impl<N: XmlNode> Mappable<N> for ManyMap<N> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn get_by_map_key(&self, map_key: &atomic::MapKey) -> Option<&sequence::Sequence<N>> {
        self.0.get(map_key).map(|(_, v)| v)
    }

    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_ {
        self.0.keys()
    }

    fn map_key_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence<N>)> + '_ {
        self.0.iter().map(|(k, (_, v))| (k, v))
    }

    fn full_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &Entry<N>)> + '_ {
        self.0.iter()
    }

    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_ {
        self.0.values().map(|(k, _)| k)
    }

    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence<N>)> + '_ {
        self.0.iter().map(|(_, (k, v))| (k, v))
    }
}

impl<N: XmlNode> TryFrom<Vec<Entry<N>>> for ManyMap<N> {
    type Error = error::Error;
    fn try_from(vec: Vec<Entry<N>>) -> error::Result<Self> {
        Self::new(vec)
    }
}
