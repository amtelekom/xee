use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use xee_datasource::XmlDS;

use crate::ast_core as ast;
use crate::attributes::Attributes;
use crate::combinator::NodeParser;
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::{AnalyzeStringContent, IterateContent, MergeContent, TemplateContent};
use crate::state::State;

#[derive(Clone)]
pub(crate) struct Content<'a, S: XmlDS> {
    pub(crate) node: S::Node,
    pub(crate) state: &'a State<S>,
    pub(crate) context: Context<S>,
    pub(crate) lazy_init: Arc<LazyInit<S>>,
}

// We use OnceLock to declare content parser once, and then reuse them
#[allow(type_alias_bounds)]
pub(crate) type ContentParseLock<V, S: XmlDS> = OnceLock<
    Box<dyn Fn(&Content<S>) -> Result<V, ElementError> + std::marker::Sync + std::marker::Send>,
>;
#[allow(type_alias_bounds)]
pub(crate) type NodeParserLock<V, S: XmlDS> =
    OnceLock<Box<dyn NodeParser<V, S> + std::marker::Sync + std::marker::Send>>;

#[derive(Default)]
pub(crate) struct LazyInit<S: XmlDS> {
    pub(crate) sequence_constructor: ContentParseLock<ast::SequenceConstructor, S>,
    pub(crate) sequence_constructor_content: ContentParseLock<ast::SequenceConstructor, S>,
    pub(crate) declarations_content: ContentParseLock<ast::Declarations, S>,

    pub(crate) accumulator_content: ContentParseLock<Vec<ast::AccumulatorRule>, S>,
    pub(crate) analyze_string: ContentParseLock<AnalyzeStringContent, S>,
    pub(crate) apply_imports_content: ContentParseLock<Vec<ast::WithParam>, S>,
    pub(crate) apply_templates_content: ContentParseLock<Vec<ast::ApplyTemplatesContent>, S>,
    pub(crate) attribute_set_content: ContentParseLock<Vec<ast::Attribute>, S>,
    pub(crate) call_template_content: ContentParseLock<Vec<ast::WithParam>, S>,
    pub(crate) character_map_content: ContentParseLock<Vec<ast::OutputCharacter>, S>,
    pub(crate) choose_content: ContentParseLock<(Vec<ast::When>, Option<ast::Otherwise>), S>,
    pub(crate) evaluate_content: ContentParseLock<Vec<ast::EvaluateContent>, S>,
    pub(crate) for_each_content: ContentParseLock<(Vec<ast::Sort>, ast::SequenceConstructor), S>,
    pub(crate) for_each_group_content:
        ContentParseLock<(Vec<ast::Sort>, ast::SequenceConstructor), S>,
    pub(crate) fork_content: ContentParseLock<(Vec<ast::Fallback>, ast::ForkContent), S>,
    pub(crate) function_content: ContentParseLock<(Vec<ast::Param>, ast::SequenceConstructor), S>,
    pub(crate) merge_content: ContentParseLock<MergeContent, S>,
    pub(crate) iterate_content: ContentParseLock<IterateContent, S>,
    pub(crate) merge_source_content: ContentParseLock<Vec<ast::MergeKey>, S>,
    pub(crate) next_iteration_content: ContentParseLock<Vec<ast::WithParam>, S>,
    pub(crate) next_match_content: ContentParseLock<Vec<ast::NextMatchContent>, S>,
    pub(crate) override_content: ContentParseLock<Vec<ast::OverrideContent>, S>,
    pub(crate) template_content: ContentParseLock<TemplateContent, S>,
}

impl<'a, S: XmlDS> Content<'a, S> {
    pub(crate) fn new(node: S::Node, state: &'a State<S>, context: Context<S>) -> Self {
        static LAZY_INITS: Mutex<
            OnceCell<HashMap<std::any::TypeId, Arc<dyn std::any::Any + Sync + Send>>>,
        > = Mutex::new(OnceCell::new());
        // Store one set of LAZY_INITS per XmlDS:
        let mut inits = LAZY_INITS.lock().unwrap();
        // while get_mut_or_init is not stabilized (rust#121641), do it in 2 steps:
        inits.get_or_init(Default::default);
        let lazy_init = inits
            .get_mut()
            .unwrap()
            .entry(std::any::TypeId::of::<S>())
            .or_insert_with(|| Arc::new(LazyInit::<S>::default()))
            .clone()
            .downcast::<LazyInit<S>>()
            .unwrap();

        Self {
            node,
            state,
            context,
            lazy_init,
        }
    }

    pub(crate) fn with_context(&self, context: Context<S>) -> Self {
        Self {
            context,
            ..self.clone()
        }
    }

    pub(crate) fn with_node(&self, node: S::Node) -> Self {
        Self {
            node,
            ..self.clone()
        }
    }

    pub(crate) fn attributes(self, element: &'a S::Element) -> Attributes<'a, S> {
        Attributes::new(self, element)
    }

    pub(crate) fn source_namespaces(&self) -> S::NamespaceAccess<'_> {
        self.state.source.namespaces(self.node)
    }

    pub(crate) fn xot_attributes(&self) -> S::AttribAccess<'_> {
        self.state.source.attributes(self.node)
    }
}
