use xee_datasource::XmlNodeWrapper;
use xee_interpreter::function::XmlDS;
use xee_interpreter::sequence::Item;

use crate::{error::Result, DocumentHandle, Documents};

// TODO: if the underlying APIs take a sequence we could turn this into
// a sequenceable.

/// Something that can be converted into an [`Item`] using a [`Document`]
///
/// This can be an item, but also a [`DocumentHandle`]
pub trait Itemable<S: XmlDS> {
    /// Convert this itemable into an [`Item`]
    fn to_item(&self, documents: &Documents<S>) -> Result<Item<S::Node>>;
}

impl<S: XmlDS> Itemable<S> for XmlNodeWrapper<S::Node> {
    fn to_item(&self, _documents: &Documents<S>) -> Result<Item<S::Node>> {
        Ok(Item::Node(self.0))
    }
}

impl<S: XmlDS> Itemable<S> for DocumentHandle {
    fn to_item(&self, documents: &Documents<S>) -> Result<Item<S::Node>> {
        // TODO: This unwrap is not great; we should turn this into an error
        let documents_ref = documents.documents.borrow();
        let document = documents_ref.get_by_handle(*self).unwrap();
        Ok(Item::Node(document.root()))
    }
}

impl<S: XmlDS> Itemable<S> for &Item<S::Node> {
    fn to_item(&self, _documents: &Documents<S>) -> Result<Item<S::Node>> {
        Ok(Clone::clone(self))
    }
}
