use std::marker::PhantomData;

use xee_datasource::XmlDS;

use crate::content::Content;
use crate::context::Context;
use crate::error::ElementError;
use crate::state::State;

type Result<V> = std::result::Result<V, ElementError>;

pub(crate) trait NodeParser<V, S: XmlDS> {
    fn parse(&self, node: Option<S::Node>, state: &State<S>, context: &Context<S>) -> Result<V> {
        let (item, next) = self.parse_next(node, state, context)?;
        if let Some(next) = next {
            // we shouldn't have any next item at this point
            Err(ElementError::Unexpected {
                span: state.span(next).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(item)
        }
    }

    fn parse_content(&self, content: Content<S>) -> Result<V> {
        self.parse(Some(content.node), content.state, &content.context)
    }

    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(V, Option<S::Node>)>;

    fn map<O, M>(self, map_func: M) -> MapParser<V, O, Self, M, S>
    where
        Self: Sized,
        M: Fn(V) -> O,
    {
        MapParser {
            parser: self,
            map_func,
            v: std::marker::PhantomData,
            o: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn option(self) -> OptionParser<V, Self, S>
    where
        Self: Sized,
    {
        OptionParser {
            parser: self,
            v: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn many(self) -> ManyParser<V, Self, S>
    where
        Self: Sized,
    {
        ManyParser {
            parser: self,
            v: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn one_or_more(self) -> OneOrMoreParser<V, Self, S>
    where
        Self: Sized,
    {
        OneOrMoreParser {
            parser: ManyParser {
                parser: self,
                v: std::marker::PhantomData,
                _source: PhantomData,
            },
        }
    }

    fn flatten<T>(self) -> FlattenParser<T, Self, S>
    where
        Self: Sized + NodeParser<Vec<T>, S>,
    {
        FlattenParser {
            parser: ManyParser {
                parser: self,

                v: std::marker::PhantomData,
                _source: PhantomData,
            },
        }
    }

    fn contains(self) -> ContainsParser<V, Self, S>
    where
        Self: Sized,
    {
        ContainsParser {
            parser: self,
            t: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn then<B, O: NodeParser<B, S>>(self, other: O) -> CombinedParser<V, B, Self, O, S>
    where
        Self: Sized,
    {
        CombinedParser {
            first: self,
            second: other,
            ta: std::marker::PhantomData,
            tb: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn then_ignore<B, O: NodeParser<B, S>>(
        self,
        other: O,
    ) -> IgnoreRightCombinedParser<V, B, Self, O, S>
    where
        Self: Sized,
    {
        IgnoreRightCombinedParser {
            first: self,
            second: other,
            ta: std::marker::PhantomData,
            tb: std::marker::PhantomData,
            _source: PhantomData,
        }
    }

    fn or<O: NodeParser<V, S>>(self, other: O) -> OrParser<V, Self, O, S>
    where
        Self: Sized,
    {
        OrParser {
            first: self,
            second: other,
            t: std::marker::PhantomData,
            _source: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OneParser<V, P, S: XmlDS>
where
    P: Fn(Content<S>) -> Result<V>,
{
    parse_value: P,
    _source: PhantomData<S>,
}

pub(crate) fn one<V, P, S: XmlDS>(parse_value: P) -> OneParser<V, P, S>
where
    P: Fn(Content<S>) -> Result<V>,
{
    OneParser {
        parse_value,
        _source: PhantomData,
    }
}

impl<V, P, S: XmlDS> NodeParser<V, S> for OneParser<V, P, S>
where
    P: Fn(Content<S>) -> Result<V>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(V, Option<S::Node>)> {
        if let Some(node) = node {
            let content = Content::new(node, state, context.clone());
            let item = (self.parse_value)(content)?;
            Ok((item, state.next(node)))
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MultiParser<V, P, S: XmlDS>
where
    P: Fn(Content<S>) -> Result<Vec<V>>,
{
    parse_value: P,
    _source: PhantomData<S>,
}

pub(crate) fn multi<V, P, S: XmlDS>(parse_value: P) -> MultiParser<V, P, S>
where
    P: Fn(Content<S>) -> Result<Vec<V>>,
{
    MultiParser {
        parse_value,
        _source: PhantomData,
    }
}

impl<V, P, S: XmlDS> NodeParser<Vec<V>, S> for MultiParser<V, P, S>
where
    P: Fn(Content<S>) -> Result<Vec<V>>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(Vec<V>, Option<S::Node>)> {
        if let Some(node) = node {
            let content = Content::new(node, state, context.clone());
            let items = (self.parse_value)(content)?;
            Ok((items, state.next(node)))
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MapParser<V, O, P, M, S: XmlDS>
where
    P: NodeParser<V, S>,
    M: Fn(V) -> O,
{
    parser: P,
    map_func: M,
    v: std::marker::PhantomData<V>,
    o: std::marker::PhantomData<O>,
    _source: PhantomData<S>,
}

impl<V, O, P, M, S: XmlDS> NodeParser<O, S> for MapParser<V, O, P, M, S>
where
    P: NodeParser<V, S>,
    M: Fn(V) -> O,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(O, Option<S::Node>)> {
        let (item, next) = self.parser.parse_next(node, state, context)?;
        Ok(((self.map_func)(item), next))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OptionParser<V, P, S: XmlDS>
where
    P: NodeParser<V, S>,
{
    parser: P,
    v: std::marker::PhantomData<V>,
    _source: PhantomData<S>,
}

impl<V, P, S: XmlDS> NodeParser<Option<V>, S> for OptionParser<V, P, S>
where
    P: NodeParser<V, S>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(Option<V>, Option<S::Node>)> {
        match self.parser.parse_next(node, state, context) {
            Ok((item, next)) => Ok((Some(item), next)),
            Err(ElementError::Unexpected { .. }) => Ok((None, node)),
            Err(ElementError::UnexpectedEnd) => Ok((None, None)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ManyParser<V, P, S: XmlDS>
where
    P: NodeParser<V, S>,
{
    parser: P,
    v: std::marker::PhantomData<V>,
    _source: PhantomData<S>,
}

impl<V, P, S: XmlDS> NodeParser<Vec<V>, S> for ManyParser<V, P, S>
where
    P: NodeParser<V, S>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(Vec<V>, Option<S::Node>)> {
        let mut result = Vec::new();
        let mut current_node = node;
        loop {
            match self.parser.parse_next(current_node, state, context) {
                Ok((item, next)) => {
                    result.push(item);
                    current_node = next;
                }
                Err(ElementError::UnexpectedEnd) => {
                    return Ok((result, None));
                }
                Err(ElementError::Unexpected { .. }) => {
                    return Ok((result, current_node));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OneOrMoreParser<V, P, S: XmlDS>
where
    P: NodeParser<V, S>,
{
    parser: ManyParser<V, P, S>,
}

impl<V, P, S: XmlDS> NodeParser<Vec<V>, S> for OneOrMoreParser<V, P, S>
where
    P: NodeParser<V, S>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(Vec<V>, Option<S::Node>)> {
        let (items, next) = self.parser.parse_next(node, state, context)?;
        if !items.is_empty() {
            Ok((items, next))
        } else if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlattenParser<V, P, S: XmlDS>
where
    P: NodeParser<Vec<V>, S>,
{
    parser: ManyParser<Vec<V>, P, S>,
}

impl<V, P, S: XmlDS> NodeParser<Vec<V>, S> for FlattenParser<V, P, S>
where
    P: NodeParser<Vec<V>, S>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(Vec<V>, Option<S::Node>)> {
        let (items, next) = self.parser.parse_next(node, state, context)?;
        let items = items.into_iter().flatten().collect();
        Ok((items, next))
    }
}

pub(crate) struct EndParser;

impl EndParser {
    pub(crate) fn new() -> Self {
        Self
    }
}

pub(crate) fn end() -> EndParser {
    EndParser::new()
}

impl<S: XmlDS> NodeParser<(), S> for EndParser {
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        _context: &Context<S>,
    ) -> Result<((), Option<S::Node>)> {
        if let Some(node) = node {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        } else {
            Ok(((), None))
        }
    }
}

pub(crate) struct CombinedParser<VA, VB, PA: NodeParser<VA, S>, PB: NodeParser<VB, S>, S: XmlDS> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<VA>,
    tb: std::marker::PhantomData<VB>,
    _source: PhantomData<S>,
}

impl<VA, VB, PA: NodeParser<VA, S>, PB: NodeParser<VB, S>, S: XmlDS> NodeParser<(VA, VB), S>
    for CombinedParser<VA, VB, PA, PB, S>
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<((VA, VB), Option<S::Node>)> {
        let (a, node) = self.first.parse_next(node, state, context)?;
        let (b, node) = self.second.parse_next(node, state, context)?;
        Ok(((a, b), node))
    }
}

pub(crate) struct IgnoreRightCombinedParser<
    VA,
    VB,
    PA: NodeParser<VA, S>,
    PB: NodeParser<VB, S>,
    S: XmlDS,
> {
    first: PA,
    second: PB,
    ta: std::marker::PhantomData<VA>,
    tb: std::marker::PhantomData<VB>,
    _source: PhantomData<S>,
}

impl<VA, VB, PA: NodeParser<VA, S>, PB: NodeParser<VB, S>, S: XmlDS> NodeParser<VA, S>
    for IgnoreRightCombinedParser<VA, VB, PA, PB, S>
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(VA, Option<S::Node>)> {
        let (a, node) = self.first.parse_next(node, state, context)?;
        let (_b, node) = self.second.parse_next(node, state, context)?;
        Ok((a, node))
    }
}

pub(crate) struct OrParser<V, PA: NodeParser<V, S>, PB: NodeParser<V, S>, S: XmlDS> {
    first: PA,
    second: PB,
    t: std::marker::PhantomData<V>,
    _source: PhantomData<S>,
}

impl<V, PA: NodeParser<V, S>, PB: NodeParser<V, S>, S: XmlDS> NodeParser<V, S>
    for OrParser<V, PA, PB, S>
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(V, Option<S::Node>)> {
        // try the first parser, if that works, return result
        // if it isn't working, try the other parser
        let r = self.first.parse_next(node, state, context);
        if r.is_ok() {
            r
        } else {
            let r2 = self.second.parse_next(node, state, context);
            if r2.is_ok() {
                r2
            } else {
                match r2 {
                    Err(ElementError::Unexpected { .. }) => r,
                    _ => r2,
                }
            }
        }
    }
}

pub(crate) struct ContainsParser<V, P, S: XmlDS>
where
    P: NodeParser<V, S>,
{
    parser: P,
    t: std::marker::PhantomData<V>,
    _source: PhantomData<S>,
}

impl<V, P, S: XmlDS> NodeParser<V, S> for ContainsParser<V, P, S>
where
    P: NodeParser<V, S>,
{
    fn parse_next(
        &self,
        node: Option<S::Node>,
        state: &State<S>,
        context: &Context<S>,
    ) -> Result<(V, Option<S::Node>)> {
        if let Some(node) = node {
            self.parser
                .parse_next(state.source.first_child(node), state, context)
        } else {
            Err(ElementError::UnexpectedEnd)
        }
    }
}

#[cfg(test)]
mod tests {
    use xot::NameId;
    use xot::Node;
    use xot::Xot;

    use crate::ast_core::Span;
    use crate::error::AttributeError;
    use crate::names::Names;

    use super::*;

    fn parse_base(s: &str) -> (State<Xot>, Context<Xot>, Node) {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let (doc, span_info) = xot.parse_with_span_info(s).unwrap();
        let outer = xot.document_element(doc).unwrap();
        let prefixes = xot.prefixes(outer);
        let state = State::new(xot, span_info.into(), names);
        let context = Context::new(prefixes);
        (state, context, outer)
    }

    fn parse_next(s: &str) -> (State<Xot>, Context<Xot>, Option<Node>) {
        let (state, context, outer) = parse_base(s);
        let next = state.source.first_child(outer);
        (state, context, next)
    }

    #[test]
    fn test_one() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_content| Ok(Value));

        let (item, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, Value);
        assert_eq!(next, None);
    }

    #[test]
    fn test_one_no_node() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_content| Ok(Value));

        let r = parser.parse_next(next, &state, &context);
        assert_eq!(r, Err(ElementError::UnexpectedEnd));
    }

    #[test]
    fn test_one_wrong_node() {
        let (mut state, context, next) = parse_next("<outer><b/></outer>");

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|content| {
            if let Some(element) = state.source.element(content.node) {
                if element.name() == names.name_a {
                    return Ok(Value);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(content.node).ok_or(ElementError::Internal)?,
            })
        });

        let r = parser.parse_next(next, &state, &context);
        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_option_present() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|_content| Ok(Value)).option();

        let (item, next) = option_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(item, Some(Value));
        assert_eq!(next, None);
    }

    #[test]
    fn test_option_present_but_parse_error() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|_content| {
            Err(AttributeError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0),
            }
            .into())
        })
        .option();

        let r: Result<(Option<Value>, Option<Node>)> =
            option_parser.parse_next(next, &state, &context);

        assert_eq!(
            r,
            Err(AttributeError::Invalid {
                value: "".to_string(),
                span: Span::new(0, 0)
            }
            .into())
        );
    }

    #[test]
    fn test_option_unexpected_node() {
        let (state, context, node) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = one(|Content { node, state, .. }| {
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item, next): (Option<Value>, Option<Node>) =
            optional_parser.parse_next(node, &state, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, node);
    }

    #[test]
    fn test_option_not_present() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let optional_parser = one(|_content| Ok(Value)).option();

        let (item, next) = optional_parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, None);
        assert_eq!(next, None);
    }

    #[derive(Debug, PartialEq)]
    struct ValueA;
    #[derive(Debug, PartialEq)]
    struct ValueB;

    struct TestNames {
        name_a: NameId,
        name_b: NameId,
        foo: NameId,
    }

    impl TestNames {
        fn new(xot: &mut Xot) -> Self {
            Self {
                name_a: xot.add_name("a"),
                name_b: xot.add_name("b"),
                foo: xot.add_name("foo"),
            }
        }
    }

    fn parse_two_option(
        state: &State<Xot>,
        context: &Context<Xot>,
        names: &TestNames,
        next: Option<Node>,
    ) -> Result<(Option<ValueA>, Option<ValueB>)> {
        let optional_parser_a = one(|Content { node, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item_a, next) = optional_parser_a.parse_next(next, state, context).unwrap();

        let optional_parser_b = one(|Content { node, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item_b, next) = optional_parser_b.parse_next(next, state, context).unwrap();

        let end_parser = EndParser::new();
        end_parser.parse_next(next, state, context)?;
        Ok((item_a, item_b))
    }

    #[test]
    fn test_two_option_both_present() {
        let (mut state, context, next) = parse_next("<outer><a /><b /></outer>");
        let names = TestNames::new(&mut state.source);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_option_only_a_present() {
        let (mut state, context, next) = parse_next("<outer><a /></outer>");
        let names = TestNames::new(&mut state.source);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, Some(ValueA));
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_option_only_b_present() {
        let (mut state, context, next) = parse_next("<outer><b /></outer>");
        let names = TestNames::new(&mut state.source);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, Some(ValueB));
    }

    #[test]
    fn test_two_option_neither_present() {
        let (mut state, context, next) = parse_next("<outer></outer>");

        let names = TestNames::new(&mut state.source);

        let (item_a, item_b) = parse_two_option(&state, &context, &names, next).unwrap();
        assert_eq!(item_a, None);
        assert_eq!(item_b, None);
    }

    #[test]
    fn test_two_option_unexpected() {
        let (mut state, context, next) = parse_next("<outer><c /></outer>");
        let names = TestNames::new(&mut state.source);

        let r = parse_two_option(&state, &context, &names, next);
        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_end_found() {
        let (state, context, next) = parse_next("<outer></outer>");

        let end_parser = end();

        let r = end_parser.parse_next(next, &state, &context);

        assert!(r.is_ok());
    }

    #[test]
    fn test_end_not_found() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        let end_parser = end();

        let r = end_parser.parse_next(next, &state, &context);

        assert_eq!(
            r,
            Err(ElementError::Unexpected {
                span: Span::new(8, 9)
            })
        );
    }

    #[test]
    fn test_many() {
        let (state, context, next) = parse_next("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|_content| Ok(Value)).many();

        let (items, next) = many_parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_empty() {
        let (state, context, next) = parse_next("<outer></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|_content| Ok(Value)).many();

        let (items, next) = many_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(items, vec![]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_option_then_many() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let (optional_item, next) = option_parser.parse_next(next, &state, &context).unwrap();
        let (many_items, next) = many_parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|Content { node, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let combined = option_parser.then(many_parser);

        let ((optional_item, many_items), next) =
            combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(optional_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_3_values() {
        let (mut state, context, next) = parse_next("<outer><a /><b /><b /></outer>");

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value;

        let option_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_a {
                    return Ok(ValueA);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let many_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let end_parser = end();

        let combined = option_parser.then(many_parser).then(end_parser);

        let (((option_item, many_items), _), next) =
            combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(option_item, Some(ValueA));
        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_combine_then_ignore() {
        let (mut state, context, next) = parse_next("<outer><b /><b /></outer>");

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value;

        let many_parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(ValueB);
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .many();

        let end_parser = end();

        let combined = many_parser.then_ignore(end_parser);

        let (many_items, next) = combined.parse_next(next, &state, &context).unwrap();

        assert_eq!(many_items, vec![ValueB, ValueB]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_attribute() {
        let (mut state, context, next) = parse_next(r#"<outer><b foo="FOO"/></outer>"#);

        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        struct Value {
            foo: String,
        }

        let parser = one(|Content::<Xot> { node, state, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    let value = state.source.attributes(node).get(names.foo).unwrap();
                    return Ok(Value {
                        foo: value.to_string(),
                    });
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        })
        .option();

        let (item, _next) = parser.parse_next(next, &state, &context).unwrap();

        assert_eq!(
            item,
            Some(Value {
                foo: "FOO".to_string()
            })
        );
    }

    #[test]
    fn test_contains() {
        let (state, context, outer) = parse_base("<outer><a /><a /><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value;

        let parser = one(|_content| Ok(Value)).many().contains();

        let (items, next) = parser.parse_next(Some(outer), &state, &context).unwrap();
        assert_eq!(items, vec![Value, Value, Value]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_or() {
        let (mut state, context, next) = parse_next("<outer><a/><b/></outer>");
        let names = TestNames::new(&mut state.source);

        #[derive(Debug, PartialEq)]
        enum AnyValue {
            A(ValueA),
            B(ValueB),
        }

        let parser_a = one(|Content { node, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_a {
                    return Ok(AnyValue::A(ValueA));
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        });

        let parser_b = one(|Content { node, .. }| {
            if let Some(element) = state.source.element(node) {
                if element.name() == names.name_b {
                    return Ok(AnyValue::B(ValueB));
                }
            }
            Err(ElementError::Unexpected {
                span: state.span(node).ok_or(ElementError::Internal)?,
            })
        });

        let parser = parser_a.or(parser_b).many();

        let (items, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], AnyValue::A(ValueA));
        assert_eq!(items[1], AnyValue::B(ValueB));
        assert_eq!(next, None);
    }

    #[test]
    fn test_map() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value(usize);

        #[derive(Debug, PartialEq)]
        struct Value2(usize);

        let parser = one(|_content| Ok(Value(1))).map(|item| Value2(item.0 + 1));

        let (item, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, Value2(2));
        assert_eq!(next, None);
    }

    #[test]
    fn test_multi() {
        let (state, context, next) = parse_next("<outer><a /></outer>");

        #[derive(Debug, PartialEq)]
        struct Value(usize);

        let parser = multi(|_content| Ok(vec![Value(1), Value(2)]));

        let (item, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(item, vec![Value(1), Value(2)]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_many_flatten() {
        let (state, context, next) = parse_next("<outer><a/><a/></outer>");

        #[derive(Debug, PartialEq)]
        struct Value(usize);

        let parser = one(|_content| Ok(vec![Value(1)])).flatten();

        let (items, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items, vec![Value(1), Value(1)]);
        assert_eq!(next, None);
    }

    #[test]
    fn test_multi_flatten() {
        let (state, context, next) = parse_next("<outer><a/><a/></outer>");

        #[derive(Debug, PartialEq)]
        struct Value(usize);

        let parser = multi(|_content| Ok(vec![Value(1), Value(2)])).flatten();

        let (items, next) = parser.parse_next(next, &state, &context).unwrap();
        assert_eq!(items, vec![Value(1), Value(2), Value(1), Value(2)]);
        assert_eq!(next, None);
    }
}
