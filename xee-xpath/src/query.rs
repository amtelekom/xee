//! Queries you can execute against a document.

use std::marker::PhantomData;
use std::rc::Rc;

use xee_interpreter::context::{self, StaticContext};
use xee_interpreter::error::SpannedResult as Result;
use xee_interpreter::function::XmlDS;
use xee_interpreter::interpreter::Program;
use xee_interpreter::sequence::{Item, Sequence};

use crate::{Documents, Itemable};

// import only for documentation purposes
#[cfg(doc)]
use crate::context::DynamicContextBuilder;
#[cfg(doc)]
use crate::Queries;

/// A query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait Query<V, S: XmlDS> {
    /// Get the program for the query
    fn program(&self) -> &Program<S>;

    /// Get the static context for the query.
    fn static_context(&self) -> &StaticContext<S> {
        self.program().static_context()
    }

    /// Execute the query against a dynamic context
    ///
    /// You can construct one using a [`DynamicContextBuilder`]
    fn execute_with_context(
        &self,
        documents: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<V>;

    /// Get a dynamic context builder for the query, configured with the
    /// query's static context and the document's documents.
    ///
    /// You can use this if you want to construct your own dynamic context
    /// to use with `execute_with_context`.
    fn dynamic_context_builder(
        &self,
        documents: &Documents<S>,
    ) -> context::DynamicContextBuilder<'_, S> {
        let mut context = self.program().dynamic_context_builder();
        context.documents(documents.documents().clone());
        context
    }

    /// Map the the result of the query to a different type.
    ///
    /// You need to provide a function that takes the result of the query,
    /// the document, and the item, and returns a new result.
    fn map<T, F>(self, f: F) -> MapQuery<V, T, Self, F, S>
    where
        Self: Sized,
        F: Fn(V, &mut Documents<S>, &context::DynamicContext<S>) -> Result<T> + Clone,
    {
        MapQuery {
            query: self,
            f,
            v: std::marker::PhantomData,
            t: std::marker::PhantomData,
            _source: std::marker::PhantomData,
        }
    }

    /// Excute the query against an itemable
    fn execute(&self, documents: &mut Documents<S>, item: impl Itemable<S>) -> Result<V> {
        let context_item = item.to_item(documents)?;
        self.execute_build_context(documents, move |builder| {
            builder.context_item(context_item);
        })
    }

    /// Execute a query with a specific dynamic context.
    ///
    /// This is useful if you want to build a dynamic context with specific
    /// settings (such as variables), and then execute a query against it.
    fn execute_build_context(
        &self,
        documents: &mut Documents<S>,
        build: impl FnOnce(&mut context::DynamicContextBuilder<S>),
    ) -> Result<V> {
        let mut dynamic_context_builder = self.dynamic_context_builder(documents);
        build(&mut dynamic_context_builder);
        let context = dynamic_context_builder.build();
        self.execute_with_context(documents, &context)
    }
}

/// A recursive query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait RecurseQuery<C, V, S: XmlDS> {
    /// Get the program for the query
    fn program(&self) -> &Program<S>;

    /// Get the static context for the query.
    fn static_context(&self) -> &StaticContext<S> {
        self.program().static_context()
    }

    /// Execute the query against an itemable, with context.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<C>;

    /// Get a dynamic context builder for the query, configured with the
    /// query's static context and the document's documents.
    ///
    /// You can use this if you want to construct your own dynamic context
    /// to use with `execute_with_context`.
    fn dynamic_context_builder(
        &self,
        document: &Documents<S>,
    ) -> context::DynamicContextBuilder<'_, S> {
        let mut context = self.program().dynamic_context_builder();
        context.documents(document.documents.clone());
        context
    }

    /// Execute the query against an itemable.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    fn execute(
        &self,
        document: &mut Documents<S>,
        item: &Item<S::Node>,
        recurse: &Recurse<V, S>,
    ) -> Result<C> {
        self.execute_build_context(document, recurse, |builder| {
            builder.context_item(item.clone());
        })
    }

    /// Execute a query against an itemable, building the context.
    ///
    /// This is useful if you want to build a dynamic context with specific
    /// settings (such as variables), and then execute a query against it.
    fn execute_build_context(
        &self,
        document: &mut Documents<S>,
        recurse: &Recurse<V, S>,
        build: impl FnOnce(&mut context::DynamicContextBuilder<S>),
    ) -> Result<C> {
        let mut dynamic_context_builder = self.dynamic_context_builder(document);
        build(&mut dynamic_context_builder);
        let context = dynamic_context_builder.build();
        self.execute_with_context(document, &context, recurse)
    }
}

/// This is the core conversion function that can be used to turn
/// an item that results from an XPath query into something useful
/// in Rust.
///
/// Given a [`Documents`] and an [`Item`], convert the item to a value of type `V`
pub trait Convert<V, S: XmlDS>: Fn(&mut Documents<S>, &Item<S::Node>) -> Result<V> {}
impl<V, T, S: XmlDS> Convert<V, S> for T where T: Fn(&mut Documents<S>, &Item<S::Node>) -> Result<V> {}

// Recursion was very hard to get right. The trick is to use an intermediate
// struct.
// https://stackoverflow.com/questions/16946888/is-it-possible-to-make-a-recursive-closure-in-rust

// The dyn and reference are unavoidable, as closures are not allowed
// to refer to themselves:
// https://github.com/rust-lang/rust/issues/46062
#[allow(type_alias_bounds)] //even though it is not enforced, the bound is required so we can refer to the ::Node associated
type RecurseFn<'s, V, S: XmlDS> =
    &'s dyn Fn(&mut Documents<S>, &Item<S::Node>, &Recurse<'s, V, S>) -> Result<V>;

/// An object that can be used to use a conversion function recursively.
pub struct Recurse<'s, V, S: XmlDS> {
    f: RecurseFn<'s, V, S>,
}

impl<'s, V, S: XmlDS> Recurse<'s, V, S> {
    /// Create a new recurse object given a conversion function.
    pub fn new(f: RecurseFn<'s, V, S>) -> Self {
        Self { f }
    }

    /// Execute the conversion function against an item.
    pub fn execute(&self, document: &mut Documents<S>, item: &Item<S::Node>) -> Result<V> {
        (self.f)(document, item, self)
    }
}

/// This is a query that expects a sequence that contains exactly one single item.
///
/// Construct this using [`Queries::one`].
///
/// If it's empty or has more than one item, an error is returned.
///
/// The resulting item is converted into a Rust value using the `convert` function
/// when constructing this query.
///
/// This is useful if you expect a single item to be returned from an XPath query.
#[derive(Debug, Clone)]
pub struct OneQuery<V, F, S: XmlDS>
where
    F: Convert<V, S>,
{
    pub(crate) program: Rc<Program<S>>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F, S: XmlDS> OneQuery<V, F, S>
where
    F: Convert<V, S>,
{
    /// Execute the query against a context
    pub fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<V> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let item = sequence.one()?;
        (self.convert)(document, &item)
    }
}

impl<V, F, S: XmlDS> Query<V, S> for OneQuery<V, F, S>
where
    F: Convert<V, S>,
{
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<V> {
        OneQuery::execute_with_context(self, document, context)
    }
}

/// A recursive query that expects a single item as a result.
#[derive(Debug, Clone)]
pub struct OneRecurseQuery<S: XmlDS> {
    pub(crate) program: Rc<Program<S>>,
}

impl<S: XmlDS> OneRecurseQuery<S> {
    /// Execute the query against an itemable, with context.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_context<V>(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<V> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let item = sequence.one()?;
        recurse.execute(document, &item)
    }
}

impl<V, S: XmlDS> RecurseQuery<V, V, S> for OneRecurseQuery<S> {
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<V> {
        OneRecurseQuery::execute_with_context(self, document, context, recurse)
    }
}

/// This is a query that expects an optional single item.
///
/// Construct this using ['Queries::option'].
///
/// If the sequence is empty, `None` is returned. If it contains more than one
/// item, an error is returned.
///
/// The result is converted into a Rust value using the `convert` function
/// when constructing this query.
///
/// This is useful if you expect an optional single item to be returned from an XPath query.
#[derive(Debug, Clone)]
pub struct OptionQuery<V, F, S: XmlDS>
where
    F: Convert<V, S>,
{
    pub(crate) program: Rc<Program<S>>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F, S: XmlDS> OptionQuery<V, F, S>
where
    F: Convert<V, S>,
{
    /// Execute the query against an itemable, with explicit
    /// dynamic context.
    pub fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Option<V>> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let item = sequence.option()?;
        item.map(|item| (self.convert)(document, &item)).transpose()
    }
}

impl<V, F, S: XmlDS> Query<Option<V>, S> for OptionQuery<V, F, S>
where
    F: Convert<V, S>,
{
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Option<V>> {
        Self::execute_with_context(self, document, context)
    }
}

/// A recursive query that expects an optional single item.
#[derive(Debug, Clone)]
pub struct OptionRecurseQuery<S: XmlDS> {
    pub(crate) program: Rc<Program<S>>,
}

impl<S: XmlDS> OptionRecurseQuery<S> {
    /// Execute the recursive query against an explicit dynamic context.
    pub fn execute_with_context<V>(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<Option<V>> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let item = sequence.option()?;
        item.map(|item| recurse.execute(document, &item))
            .transpose()
    }
}

impl<V, S: XmlDS> RecurseQuery<Option<V>, V, S> for OptionRecurseQuery<S> {
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<Option<V>> {
        OptionRecurseQuery::execute_with_context(self, document, context, recurse)
    }
}

/// A query that expects many items as a result.
///
/// Construct this using [`Queries::many`].
///
/// The items are converted into Rust values using the supplied `convert` function.
///
/// This is useful if you expect many items to be returned from an XPath query.
///
/// The result is converted into a Rust value using the `convert` function
/// when constructing this query.
#[derive(Debug, Clone)]
pub struct ManyQuery<V, F, S: XmlDS>
where
    F: Convert<V, S>,
{
    pub(crate) program: Rc<Program<S>>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F, S: XmlDS> ManyQuery<V, F, S>
where
    F: Convert<V, S>,
{
    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Vec<V>> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let items = sequence
            .iter()
            .map(|item| (self.convert)(document, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

impl<V, F, S: XmlDS> Query<Vec<V>, S> for ManyQuery<V, F, S>
where
    F: Convert<V, S>,
{
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Vec<V>> {
        Self::execute_with_context(self, document, context)
    }
}

/// A recursive query that expects many items as a result.
#[derive(Debug, Clone)]
pub struct ManyRecurseQuery<S: XmlDS> {
    pub(crate) program: Rc<Program<S>>,
}

impl<S: XmlDS> ManyRecurseQuery<S> {
    /// Execute the query against an itemable, with variables.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_context<V>(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<Vec<V>> {
        let sequence = self.program.runnable(context).many(document.source_mut())?;
        let items = sequence
            .iter()
            .map(|item| recurse.execute(document, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

impl<V, S: XmlDS> RecurseQuery<Vec<V>, V, S> for ManyRecurseQuery<S> {
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
        recurse: &Recurse<V, S>,
    ) -> Result<Vec<V>> {
        ManyRecurseQuery::execute_with_context(self, document, context, recurse)
    }
}

/// A query that returns a sequence.
///
/// This query returns a [`Sequence`] object that can be used to access
/// the items in the sequence. It represents an XPath sequence. The items
/// in the sequence are not converted.
///
/// Construct this using [`Queries::sequence`].
///
/// This is useful if you want to work with the sequence directly.
#[derive(Debug, Clone)]
pub struct SequenceQuery<S: XmlDS> {
    pub(crate) program: Rc<Program<S>>,
}

impl<S: XmlDS> SequenceQuery<S> {
    /// Execute the query against an itemable with an explict dynamic context.
    pub fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Sequence<S::Node>> {
        self.program.runnable(context).many(document.source_mut())
    }
}

impl<S: XmlDS> Query<Sequence<S::Node>, S> for SequenceQuery<S> {
    fn program(&self) -> &Program<S> {
        &self.program
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<Sequence<S::Node>> {
        Self::execute_with_context(self, document, context)
    }
}

/// A query maps the result of another query to a different type.
#[derive(Debug, Clone)]
pub struct MapQuery<V, T, Q: Query<V, S> + Sized, F, S: XmlDS>
where
    F: Fn(V, &mut Documents<S>, &context::DynamicContext<S>) -> Result<T> + Clone,
{
    query: Q,
    f: F,
    v: std::marker::PhantomData<V>,
    t: std::marker::PhantomData<T>,
    _source: PhantomData<S>,
}

impl<V, T, Q, F, S: XmlDS> MapQuery<V, T, Q, F, S>
where
    Q: Query<V, S> + Sized,
    F: Fn(V, &mut Documents<S>, &context::DynamicContext<S>) -> Result<T> + Clone,
{
    /// Execute the query against an item.
    pub fn execute(&self, document: &mut Documents<S>, item: &Item<S::Node>) -> Result<T> {
        let mut dynamic_context_builder = self.query.program().dynamic_context_builder();
        dynamic_context_builder.context_item(item.clone());
        let context = dynamic_context_builder.build();
        self.execute_with_context(document, &context)
    }

    /// Execute the query against a dynamic context.
    pub fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<T> {
        let v = self.query.execute_with_context(document, context)?;
        // TODO: this isn't right. need to rewrite in terms of dynamic context too?
        (self.f)(v, document, context)
    }
}

impl<V, T, Q: Query<V, S> + Sized, F, S: XmlDS> Query<T, S> for MapQuery<V, T, Q, F, S>
where
    F: Fn(V, &mut Documents<S>, &context::DynamicContext<S>) -> Result<T> + Clone,
{
    fn program(&self) -> &Program<S> {
        self.query.program()
    }

    fn execute_with_context(
        &self,
        document: &mut Documents<S>,
        context: &context::DynamicContext<S>,
    ) -> Result<T> {
        let v = self.query.execute_with_context(document, context)?;
        (self.f)(v, document, context)
    }
}
