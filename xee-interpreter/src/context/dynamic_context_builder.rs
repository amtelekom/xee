use std::{cell::RefCell, ops::Deref, rc::Rc};

use ahash::{HashMap, HashMapExt};
use iri_string::types::{IriStr, IriString};

use crate::function::XmlDS;
use crate::{interpreter, sequence, xml};

use super::{DynamicContext, Variables};

/// A builder for constructing a [`DynamicContext`].
///
/// This needs to be supplied a [`StaticContext`] (or a reference to one) in
/// order to construct it.
///
/// You can supply a context item, documents, variables and the like in order
/// to construct a dynamic context used to execute an XPath instruction.
#[derive(Debug, Clone)]
pub struct DynamicContextBuilder<'a, S: XmlDS> {
    program: &'a interpreter::Program<S>,
    context_item: Option<sequence::Item<S::Node>>,
    documents: DocumentsRef<S>,
    variables: Variables<S>,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    default_collection: Option<sequence::Sequence<S::Node>>,
    collections: HashMap<IriString, sequence::Sequence<S::Node>>,
    default_uri_collection: Option<sequence::Sequence<S::Node>>,
    uri_collections: HashMap<IriString, sequence::Sequence<S::Node>>,
    environment_variables: HashMap<String, String>,
}

/// A shallow wrapper around a collection of XML documents
/// [`xml::Documents`]
#[derive(Debug, Clone)]
pub struct DocumentsRef<S: XmlDS>(Rc<RefCell<xml::Documents<S>>>);

impl<S: XmlDS> Deref for DocumentsRef<S> {
    type Target = RefCell<xml::Documents<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: XmlDS> From<xml::Documents<S>> for DocumentsRef<S> {
    fn from(documents: xml::Documents<S>) -> Self {
        Self(Rc::new(RefCell::new(documents)))
    }
}

impl<S: XmlDS> DocumentsRef<S> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(xml::Documents::new())))
    }
}

impl<S: XmlDS> Default for DocumentsRef<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S: XmlDS> DynamicContextBuilder<'a, S> {
    /// Construct a new `DynamicContextBuilder` with the given `StaticContext`.
    pub(crate) fn new(program: &'a interpreter::Program<S>) -> Self {
        Self {
            program,
            context_item: None,
            documents: DocumentsRef::new(),
            variables: Variables::<S>::new(),
            current_datetime: chrono::offset::Local::now().into(),
            default_collection: None,
            collections: HashMap::new(),
            default_uri_collection: None,
            uri_collections: HashMap::new(),
            environment_variables: HashMap::new(),
        }
    }

    /// Set the context item of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have no context item.
    pub fn context_item(&mut self, context_item: sequence::Item<S::Node>) -> &mut Self {
        self.context_item = Some(context_item);
        self
    }

    /// Set a node as the context item of the [`DynamicContext`].
    pub fn context_node(&mut self, node: S::Node) -> &mut Self {
        self.context_item(sequence::Item::Node(node));
        self
    }

    /// Set the documents of the [`DynamicContext`].
    ///
    /// You can give it either owned documents or a [`DocumentsRef`].
    pub fn documents(&mut self, documents: impl Into<DocumentsRef<S>>) -> &mut Self {
        self.documents = documents.into();
        self
    }

    /// Set the variables of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have no variables.
    pub fn variables(&mut self, variables: Variables<S>) -> &mut Self {
        self.variables = variables;
        self
    }

    /// Set the current datetime of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have the current datetime.
    pub fn current_datetime(
        &mut self,
        current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> &mut Self {
        self.current_datetime = current_datetime;
        self
    }

    /// Set the default collection
    pub fn default_collection(&mut self, sequence: sequence::Sequence<S::Node>) -> &mut Self {
        self.default_collection = Some(sequence);
        self
    }

    /// Set a collection
    pub fn collection(&mut self, uri: &IriStr, sequence: sequence::Sequence<S::Node>) -> &mut Self {
        self.collections.insert((*uri).into(), sequence);
        self
    }

    /// Set the default URI collection
    pub fn default_uri_collection(&mut self, uris: &[&IriStr]) -> &mut Self {
        self.default_uri_collection = Some(Self::uris_into_sequence(uris));
        self
    }

    /// Set a URI collection
    pub fn uri_collection(&mut self, uri: &IriStr, uris: &[&IriStr]) -> &mut Self {
        self.uri_collections
            .insert((*uri).into(), Self::uris_into_sequence(uris));
        self
    }

    /// Set the environment variables
    pub fn environment_variables(
        &mut self,
        environment_variables: HashMap<String, String>,
    ) -> &mut Self {
        self.environment_variables = environment_variables;
        self
    }

    /// Initialize the environment variables from the current system environment.
    pub fn initialize_env(&mut self) -> &mut Self {
        self.environment_variables = std::env::vars().collect();
        self
    }

    fn uris_into_sequence(uris: &[&IriStr]) -> sequence::Sequence<S::Node> {
        // turn the URIs into a sequence
        let items: Vec<sequence::Item<S::Node>> = uris
            .iter()
            .map(|uri| {
                let iri_string: IriString = (*uri).into();
                iri_string.into()
            })
            .collect();
        items.into()
    }

    /// Build the `DynamicContext`.
    pub fn build(&self) -> DynamicContext<'_, S> {
        DynamicContext::new(
            self.program,
            self.context_item.clone(),
            self.documents.clone(),
            self.variables.clone(),
            self.current_datetime,
            self.default_collection.clone(),
            self.collections.clone(),
            self.default_uri_collection.clone(),
            self.uri_collections.clone(),
            self.environment_variables.clone(),
        )
    }
}
