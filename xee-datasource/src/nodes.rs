use std::borrow::Cow;

use crate::XmlDS;

pub trait XmlAttribAccess<'a, S: XmlDS> {
    fn get(&self, key: S::NameId) -> Option<Cow<'a, str>>;
    fn keys(&self) -> impl Iterator<Item = S::NameId> + '_;
}

pub trait XmlMutAttribAccess<'a, S: XmlDS> {
    fn insert(&mut self, key: S::NameId, value: String) -> Option<String>;
    fn remove(&mut self, key: S::NameId) -> Option<String>;
}

pub trait XmlNamespaceAccess<'a, S: XmlDS> {
    fn iter(&self) -> impl Iterator<Item = (S::PrefixId, &'a S::NamespaceId)> + '_;
}
pub trait XmlMutNamespaceAccess<'a, S: XmlDS> {
    fn insert(&mut self, prefix: S::PrefixId, namespace: S::NamespaceId) -> Option<S::NamespaceId>;
    fn remove(&mut self, key: S::PrefixId) -> Option<S::NamespaceId>;
}
