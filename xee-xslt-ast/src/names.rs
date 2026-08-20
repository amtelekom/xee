use std::collections::BTreeMap;
use std::str::FromStr;

use ahash::HashSet;
use strum::VariantNames;
use xee_datasource::XmlDS;

use crate::ast_core::{self as ast, DeclarationName, OverrideContentName, SequenceConstructorName};
use crate::attributes::Attributes;
use crate::error::ElementError;
use crate::instruction::{DeclarationParser, OverrideContentParser, SequenceConstructorParser};

impl SequenceConstructorName {
    pub(crate) fn parse<S: XmlDS>(
        &self,
        attributes: &Attributes<S>,
    ) -> Result<ast::SequenceConstructorItem, ElementError> {
        match self {
            SequenceConstructorName::ApplyImports => {
                ast::ApplyImports::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::AnalyzeString => {
                ast::AnalyzeString::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::ApplyTemplates => {
                ast::ApplyTemplates::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Assert => {
                ast::Assert::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Attribute => {
                ast::Attribute::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Break => {
                ast::Break::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::CallTemplate => {
                ast::CallTemplate::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Choose => {
                ast::Choose::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Comment => {
                ast::Comment::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Copy => ast::Copy::parse_sequence_constructor_item(attributes),
            SequenceConstructorName::CopyOf => {
                ast::CopyOf::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Document => {
                ast::Document::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Element => {
                ast::Element::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Evaluate => {
                ast::Evaluate::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Fallback => {
                ast::Fallback::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::ForEach => {
                ast::ForEach::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::ForEachGroup => {
                ast::ForEachGroup::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Fork => ast::Fork::parse_sequence_constructor_item(attributes),
            SequenceConstructorName::If => ast::If::parse_sequence_constructor_item(attributes),
            SequenceConstructorName::Iterate => {
                ast::Iterate::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Map => ast::Map::parse_sequence_constructor_item(attributes),
            SequenceConstructorName::MapEntry => {
                ast::MapEntry::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Merge => {
                ast::Merge::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Message => {
                ast::Message::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Namespace => {
                ast::Namespace::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::NextIteration => {
                ast::NextIteration::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::NextMatch => {
                ast::NextMatch::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Number => {
                ast::Number::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::OnEmpty => {
                ast::OnEmpty::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::OnNonEmpty => {
                ast::OnNonEmpty::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::ProcessingInstruction => {
                ast::ProcessingInstruction::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Sequence => {
                ast::Sequence::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::SourceDocument => {
                ast::SourceDocument::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Text => ast::Text::parse_sequence_constructor_item(attributes),
            SequenceConstructorName::ValueOf => {
                ast::ValueOf::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::Variable => {
                ast::Variable::parse_sequence_constructor_item(attributes)
            }
            SequenceConstructorName::WherePopulated => {
                ast::WherePopulated::parse_sequence_constructor_item(attributes)
            }

            _ => Err(ElementError::Unsupported(format!(
                "Unknown sequence constructor: {self:?}",
            ))),
        }
    }

    fn names<S: XmlDS>(
        source: &mut S,
        xsl_ns: S::NamespaceId,
    ) -> BTreeMap<S::NameId, SequenceConstructorName> {
        let mut sequence_constructor_names = BTreeMap::new();
        for variant_name in Self::VARIANTS {
            let name = source.add_name_ns(variant_name, xsl_ns);
            let constructor = SequenceConstructorName::from_str(variant_name).unwrap();
            sequence_constructor_names.insert(name, constructor);
        }
        sequence_constructor_names
    }
}

impl DeclarationName {
    pub(crate) fn parse<S: XmlDS>(
        &self,
        attributes: &Attributes<S>,
    ) -> Result<ast::Declaration, ElementError> {
        match self {
            DeclarationName::Accumulator => ast::Accumulator::parse_declaration(attributes),
            DeclarationName::Template => ast::Template::parse_declaration(attributes),
            DeclarationName::Output => ast::Output::parse_declaration(attributes),
            _ => Err(ElementError::Unsupported(format!(
                "Unsupported declaration: {self:?}",
            ))),
        }
    }

    fn names<S: XmlDS>(
        source: &mut S,
        xsl_ns: S::NamespaceId,
    ) -> BTreeMap<S::NameId, DeclarationName> {
        let mut declaration_names = BTreeMap::new();
        for variant_name in Self::VARIANTS {
            let name = source.add_name_ns(variant_name, xsl_ns);
            let constructor = DeclarationName::from_str(variant_name).unwrap();
            declaration_names.insert(name, constructor);
        }
        declaration_names
    }
}

impl OverrideContentName {
    pub(crate) fn parse<S: XmlDS>(
        &self,
        attributes: &Attributes<S>,
    ) -> Result<ast::OverrideContent, ElementError> {
        use ast::OverrideContentName::*;

        match self {
            Template => ast::Template::parse_override_content(attributes),
            Function => ast::Function::parse_override_content(attributes),
            Variable => ast::Variable::parse_override_content(attributes),
            Param => ast::Param::parse_override_content(attributes),
            AttributeSet => ast::AttributeSet::parse_override_content(attributes),
        }
    }

    fn names<S: XmlDS>(
        source: &mut S,
        xsl_ns: S::NamespaceId,
    ) -> BTreeMap<S::NameId, OverrideContentName> {
        let mut override_content_names = BTreeMap::new();
        for variant_name in Self::VARIANTS {
            let name = source.add_name_ns(variant_name, xsl_ns);
            let constructor = OverrideContentName::from_str(variant_name).unwrap();
            override_content_names.insert(name, constructor);
        }
        override_content_names
    }
}

pub(crate) struct Names<S: XmlDS> {
    pub(crate) xsl_ns: S::NamespaceId,

    pub(crate) sequence_constructor_names: BTreeMap<S::NameId, SequenceConstructorName>,
    pub(crate) declaration_names: BTreeMap<S::NameId, DeclarationName>,
    pub(crate) override_content_names: BTreeMap<S::NameId, OverrideContentName>,
    pub(crate) ignore_xml_space_parents: HashSet<S::NameId>,
    pub(crate) ignore_xml_space_next_siblings: HashSet<S::NameId>,
    pub(crate) ignore_xml_space_previous_siblings: HashSet<S::NameId>,

    // XSL elements
    pub(crate) xsl_accumulator_rule: S::NameId,
    pub(crate) xsl_attribute: S::NameId,
    pub(crate) xsl_fallback: S::NameId,
    pub(crate) xsl_for_each: S::NameId,
    pub(crate) xsl_for_each_group: S::NameId,
    pub(crate) xsl_matching_substring: S::NameId,
    pub(crate) xsl_merge_action: S::NameId,
    pub(crate) xsl_merge_key: S::NameId,
    pub(crate) xsl_merge_source: S::NameId,
    pub(crate) xsl_non_matching_substring: S::NameId,
    pub(crate) xsl_on_completion: S::NameId,
    pub(crate) xsl_otherwise: S::NameId,
    pub(crate) xsl_output_character: S::NameId,
    pub(crate) xsl_param: S::NameId,
    pub(crate) xsl_schema: S::NameId,
    pub(crate) xsl_sequence: S::NameId,
    pub(crate) xsl_sort: S::NameId,
    pub(crate) xsl_text: S::NameId,
    pub(crate) xsl_transform: S::NameId,
    pub(crate) xsl_stylesheet: S::NameId,
    pub(crate) xsl_variable: S::NameId,
    pub(crate) xsl_when: S::NameId,
    pub(crate) xsl_with_param: S::NameId,

    // attributes on XSLT elements
    pub(crate) allow_duplicate_names: S::NameId,
    pub(crate) as_: S::NameId,
    pub(crate) base_uri: S::NameId,
    pub(crate) build_tree: S::NameId,
    pub(crate) byte_order_mark: S::NameId,
    pub(crate) cache: S::NameId,
    pub(crate) case_order: S::NameId,
    pub(crate) cdata_section_elements: S::NameId,
    pub(crate) character: S::NameId,
    pub(crate) component: S::NameId,
    pub(crate) composite: S::NameId,
    pub(crate) context_item: S::NameId,
    pub(crate) collation: S::NameId,
    pub(crate) copy_accumulators: S::NameId,
    pub(crate) copy_namespaces: S::NameId,
    pub(crate) count: S::NameId,
    pub(crate) data_type: S::NameId,
    pub(crate) decimal_separator: S::NameId,
    pub(crate) digit: S::NameId,
    pub(crate) disable_output_escaping: S::NameId,
    pub(crate) doctype_public: S::NameId,
    pub(crate) doctype_system: S::NameId,
    pub(crate) elements: S::NameId,
    pub(crate) encoding: S::NameId,
    pub(crate) error_code: S::NameId,
    pub(crate) errors: S::NameId,
    pub(crate) escape_uri_attributes: S::NameId,
    pub(crate) exponent_separator: S::NameId,
    pub(crate) extension_element_prefixes: S::NameId,
    pub(crate) flags: S::NameId,
    pub(crate) for_each_item: S::NameId,
    pub(crate) for_each_source: S::NameId,
    pub(crate) format: S::NameId,
    pub(crate) from: S::NameId,
    pub(crate) group_adjacent: S::NameId,
    pub(crate) group_by: S::NameId,
    pub(crate) group_ending_with: S::NameId,
    pub(crate) group_starting_with: S::NameId,
    pub(crate) grouping_size: S::NameId,
    pub(crate) grouping_separator: S::NameId,
    pub(crate) href: S::NameId,
    pub(crate) html_version: S::NameId,
    pub(crate) id: S::NameId,
    pub(crate) include_content_type: S::NameId,
    pub(crate) indent: S::NameId,
    pub(crate) infinity: S::NameId,
    pub(crate) inherit_namespaces: S::NameId,
    pub(crate) initial_value: S::NameId,
    pub(crate) input_type_annotations: S::NameId,
    pub(crate) item_separator: S::NameId,
    pub(crate) json_node_output_method: S::NameId,
    pub(crate) key: S::NameId,
    pub(crate) lang: S::NameId,
    pub(crate) letter_value: S::NameId,
    pub(crate) level: S::NameId,
    pub(crate) match_: S::NameId,
    pub(crate) media_type: S::NameId,
    pub(crate) method: S::NameId,
    pub(crate) minus_sign: S::NameId,
    pub(crate) mode: S::NameId,
    pub(crate) name: S::NameId,
    pub(crate) names: S::NameId,
    pub(crate) namespace: S::NameId,
    pub(crate) namespace_context: S::NameId,
    pub(crate) normalization_form: S::NameId,
    pub(crate) nan: S::NameId,
    pub(crate) new_each_time: S::NameId,
    pub(crate) omit_xml_declaration: S::NameId,
    pub(crate) on_multiple_match: S::NameId,
    pub(crate) on_no_match: S::NameId,
    pub(crate) order: S::NameId,
    pub(crate) ordinal: S::NameId,
    pub(crate) override_: S::NameId,
    pub(crate) override_extension_function: S::NameId,
    pub(crate) parameter_document: S::NameId,
    pub(crate) pattern_separator: S::NameId,
    pub(crate) phase: S::NameId,
    pub(crate) percent: S::NameId,
    pub(crate) per_mille: S::NameId,
    pub(crate) priority: S::NameId,
    pub(crate) regex: S::NameId,
    pub(crate) required: S::NameId,
    pub(crate) result_prefix: S::NameId,
    pub(crate) schema_aware: S::NameId,
    pub(crate) schema_location: S::NameId,
    pub(crate) select: S::NameId,
    pub(crate) separator: S::NameId,
    pub(crate) sort_before_merge: S::NameId,
    pub(crate) stable: S::NameId,
    pub(crate) standalone: S::NameId,
    pub(crate) start_at: S::NameId,
    pub(crate) static_: S::NameId,
    pub(crate) streamable: S::NameId,
    pub(crate) streamability: S::NameId,
    pub(crate) string: S::NameId,
    pub(crate) stylesheet_prefix: S::NameId,
    pub(crate) suppress_indentation: S::NameId,
    pub(crate) terminate: S::NameId,
    pub(crate) test: S::NameId,
    pub(crate) tunnel: S::NameId,
    pub(crate) type_: S::NameId,
    pub(crate) typed: S::NameId,
    pub(crate) undeclare_prefixes: S::NameId,
    pub(crate) use_: S::NameId,
    pub(crate) use_accumulators: S::NameId,
    pub(crate) use_attribute_sets: S::NameId,
    pub(crate) use_character_maps: S::NameId,
    pub(crate) validation: S::NameId,
    pub(crate) value: S::NameId,
    pub(crate) version: S::NameId,
    pub(crate) visibility: S::NameId,
    pub(crate) warning_on_multiple_match: S::NameId,
    pub(crate) warning_on_no_match: S::NameId,
    pub(crate) with_params: S::NameId,
    pub(crate) xpath: S::NameId,
    pub(crate) zero_digit: S::NameId,

    // standard attributes on XSLT elements
    pub(crate) standard: StandardNames<S>,
    // standard attributes on literal result elements
    pub(crate) xsl_standard: StandardNames<S>,
}

pub(crate) struct StandardNames<S: XmlDS> {
    pub(crate) default_collation: S::NameId,
    pub(crate) default_mode: S::NameId,
    pub(crate) default_validation: S::NameId,
    pub(crate) exclude_result_prefixes: S::NameId,
    pub(crate) expand_text: S::NameId,
    pub(crate) extension_element_prefixes: S::NameId,
    pub(crate) use_when: S::NameId,
    pub(crate) version: S::NameId,
    pub(crate) xpath_default_namespace: S::NameId,
}

impl<S: XmlDS> StandardNames<S> {
    fn no_ns(source: &mut S) -> Self {
        Self {
            default_collation: source.add_name("default-collation"),
            default_mode: source.add_name("default-mode"),
            default_validation: source.add_name("default-validation"),
            exclude_result_prefixes: source.add_name("exclude-result-prefixes"),
            expand_text: source.add_name("expand-text"),
            extension_element_prefixes: source.add_name("extension-element-prefixes"),
            use_when: source.add_name("use-when"),
            version: source.add_name("version"),
            xpath_default_namespace: source.add_name("xpath-default-namespace"),
        }
    }

    fn xsl(source: &mut S, xsl_ns: S::NamespaceId) -> Self {
        Self {
            default_collation: source.add_name_ns("default-collation", xsl_ns),
            default_mode: source.add_name_ns("default-mode", xsl_ns),
            default_validation: source.add_name_ns("default-validation", xsl_ns),
            exclude_result_prefixes: source.add_name_ns("exclude-result-prefixes", xsl_ns),
            expand_text: source.add_name_ns("expand-text", xsl_ns),
            extension_element_prefixes: source.add_name_ns("extension-element-prefixes", xsl_ns),
            use_when: source.add_name_ns("use-when", xsl_ns),
            version: source.add_name_ns("version", xsl_ns),
            xpath_default_namespace: source.add_name_ns("xpath-default-namespace", xsl_ns),
        }
    }
}

impl<S: XmlDS> Names<S> {
    pub(crate) fn new(source: &mut S) -> Self {
        let xsl_ns = source.add_namespace("http://www.w3.org/1999/XSL/Transform");

        let ignore_xml_space_parents = [
            source.add_name_ns("accumulator", xsl_ns),
            source.add_name_ns("analyze-string", xsl_ns),
            source.add_name_ns("apply-imports", xsl_ns),
            source.add_name_ns("apply-templates", xsl_ns),
            source.add_name_ns("attribute-set", xsl_ns),
            source.add_name_ns("call-template", xsl_ns),
            source.add_name_ns("character-map", xsl_ns),
            source.add_name_ns("choose", xsl_ns),
            source.add_name_ns("evaluate", xsl_ns),
            source.add_name_ns("fork", xsl_ns),
            source.add_name_ns("merge", xsl_ns),
            source.add_name_ns("merge-source", xsl_ns),
            source.add_name_ns("mode", xsl_ns),
            source.add_name_ns("next-iteration", xsl_ns),
            source.add_name_ns("next-match", xsl_ns),
            source.add_name_ns("override", xsl_ns),
            source.add_name_ns("package", xsl_ns),
            source.add_name_ns("stylesheet", xsl_ns),
            source.add_name_ns("transform", xsl_ns),
            source.add_name_ns("use-package", xsl_ns),
        ]
        .iter()
        .copied()
        .collect();

        let ignore_xml_space_next_siblings = [
            source.add_name_ns("param", xsl_ns),
            source.add_name_ns("sort", xsl_ns),
            source.add_name_ns("context-item", xsl_ns),
            source.add_name_ns("on-completion", xsl_ns),
        ]
        .iter()
        .copied()
        .collect();

        let ignore_xml_space_previous_siblings = [source.add_name_ns("catch", xsl_ns)]
            .iter()
            .copied()
            .collect();

        Self {
            xsl_ns,

            sequence_constructor_names: SequenceConstructorName::names(source, xsl_ns),
            declaration_names: DeclarationName::names(source, xsl_ns),
            override_content_names: OverrideContentName::names(source, xsl_ns),
            ignore_xml_space_parents,
            ignore_xml_space_next_siblings,
            ignore_xml_space_previous_siblings,

            xsl_accumulator_rule: source.add_name_ns("accumulator-rule", xsl_ns),
            xsl_attribute: source.add_name_ns("attribute", xsl_ns),
            xsl_fallback: source.add_name_ns("fallback", xsl_ns),
            xsl_for_each: source.add_name_ns("for-each", xsl_ns),
            xsl_for_each_group: source.add_name_ns("for-each-group", xsl_ns),
            xsl_matching_substring: source.add_name_ns("matching-substring", xsl_ns),
            xsl_merge_action: source.add_name_ns("merge-action", xsl_ns),
            xsl_merge_key: source.add_name_ns("merge-key", xsl_ns),
            xsl_merge_source: source.add_name_ns("merge-source", xsl_ns),
            xsl_non_matching_substring: source.add_name_ns("non-matching-substring", xsl_ns),
            xsl_on_completion: source.add_name_ns("on-completion", xsl_ns),
            xsl_otherwise: source.add_name_ns("otherwise", xsl_ns),
            xsl_output_character: source.add_name_ns("output-character", xsl_ns),
            xsl_param: source.add_name_ns("param", xsl_ns),
            xsl_schema: source.add_name_ns("schema", xsl_ns),
            xsl_sequence: source.add_name_ns("sequence", xsl_ns),
            xsl_sort: source.add_name_ns("sort", xsl_ns),
            xsl_text: source.add_name_ns("text", xsl_ns),
            xsl_transform: source.add_name_ns("transform", xsl_ns),
            xsl_stylesheet: source.add_name_ns("stylesheet", xsl_ns),
            xsl_variable: source.add_name_ns("variable", xsl_ns),
            xsl_when: source.add_name_ns("when", xsl_ns),
            xsl_with_param: source.add_name_ns("with-param", xsl_ns),

            as_: source.add_name("as"),
            allow_duplicate_names: source.add_name("allow-duplicate-names"),
            base_uri: source.add_name("base-uri"),
            build_tree: source.add_name("build-tree"),
            byte_order_mark: source.add_name("byte-order-mark"),
            case_order: source.add_name("case-order"),
            cache: source.add_name("cache"),
            cdata_section_elements: source.add_name("cdata-section-elements"),
            character: source.add_name("character"),
            collation: source.add_name("collation"),
            component: source.add_name("component"),
            composite: source.add_name("composite"),
            context_item: source.add_name("context-item"),
            copy_accumulators: source.add_name("copy-accumulators"),
            copy_namespaces: source.add_name("copy-namespaces"),
            count: source.add_name("count"),
            data_type: source.add_name("data-type"),
            decimal_separator: source.add_name("decimal-separator"),
            digit: source.add_name("digit"),
            disable_output_escaping: source.add_name("disable-output-escaping"),
            doctype_public: source.add_name("doctype-public"),
            doctype_system: source.add_name("doctype-system"),
            elements: source.add_name("elements"),
            encoding: source.add_name("encoding"),
            error_code: source.add_name("error-code"),
            errors: source.add_name("errors"),
            escape_uri_attributes: source.add_name("escape-uri-attributes"),
            exponent_separator: source.add_name("exponent-separator"),
            extension_element_prefixes: source.add_name("extension-element-prefixes"),
            flags: source.add_name("flags"),
            for_each_item: source.add_name("for-each-item"),
            for_each_source: source.add_name("for-each-source"),
            format: source.add_name("format"),
            from: source.add_name("from"),
            group_adjacent: source.add_name("group-adjacent"),
            group_by: source.add_name("group-by"),
            group_ending_with: source.add_name("group-ending-with"),
            group_starting_with: source.add_name("group-starting-with"),
            grouping_separator: source.add_name("grouping-separator"),
            grouping_size: source.add_name("grouping-size"),
            href: source.add_name("href"),
            html_version: source.add_name("html-version"),
            id: source.add_name("id"),
            include_content_type: source.add_name("include-content-type"),
            indent: source.add_name("indent"),
            infinity: source.add_name("infinity"),
            inherit_namespaces: source.add_name("inherit-namespaces"),
            initial_value: source.add_name("initial-value"),
            input_type_annotations: source.add_name("input-type-annotations"),
            item_separator: source.add_name("item-separator"),
            json_node_output_method: source.add_name("json-node-output-method"),
            lang: source.add_name("language"),
            letter_value: source.add_name("letter-value"),
            level: source.add_name("level"),
            key: source.add_name("key"),
            match_: source.add_name("match"),
            media_type: source.add_name("media-type"),
            method: source.add_name("method"),
            minus_sign: source.add_name("minus-sign"),
            mode: source.add_name("mode"),
            name: source.add_name("name"),
            names: source.add_name("names"),
            nan: source.add_name("NaN"),
            namespace: source.add_name("namespace"),
            namespace_context: source.add_name("namespace-context"),
            new_each_time: source.add_name("new-each-time"),
            normalization_form: source.add_name("normalization-form"),
            omit_xml_declaration: source.add_name("omit-xml-declaration"),
            on_multiple_match: source.add_name("on-multiple-match"),
            on_no_match: source.add_name("on-no-match"),
            order: source.add_name("order"),
            ordinal: source.add_name("ordinal"),
            override_: source.add_name("override"),
            override_extension_function: source.add_name("override-extension-function"),
            pattern_separator: source.add_name("pattern-separator"),
            parameter_document: source.add_name("parameter-document"),
            phase: source.add_name("phase"),
            percent: source.add_name("percent"),
            per_mille: source.add_name("per-mille"),
            priority: source.add_name("priority"),
            regex: source.add_name("regex"),
            required: source.add_name("required"),
            result_prefix: source.add_name("result-prefix"),
            schema_aware: source.add_name("schema-aware"),
            schema_location: source.add_name("schema-location"),
            select: source.add_name("select"),
            separator: source.add_name("separator"),
            sort_before_merge: source.add_name("sort-before-merge"),
            stable: source.add_name("stable"),
            standalone: source.add_name("standalone"),
            start_at: source.add_name("start-at"),
            static_: source.add_name("static"),
            streamability: source.add_name("streamability"),
            streamable: source.add_name("streamable"),
            string: source.add_name("string"),
            stylesheet_prefix: source.add_name("stylesheet-prefix"),
            suppress_indentation: source.add_name("suppress-indentation"),
            terminate: source.add_name("terminate"),
            test: source.add_name("test"),
            tunnel: source.add_name("tunnel"),
            type_: source.add_name("type"),
            typed: source.add_name("typed"),
            use_: source.add_name("use"),
            use_accumulators: source.add_name("use-accumulators"),
            use_attribute_sets: source.add_name("use-attribute-sets"),
            use_character_maps: source.add_name("use-character-maps"),
            validation: source.add_name("validation"),
            visibility: source.add_name("visibility"),
            undeclare_prefixes: source.add_name("undeclare-prefixes"),
            value: source.add_name("value"),
            version: source.add_name("version"),
            warning_on_no_match: source.add_name("warning-on-no-match"),
            warning_on_multiple_match: source.add_name("warning-on-multiple-match"),
            with_params: source.add_name("with-params"),
            xpath: source.add_name("xpath"),
            zero_digit: source.add_name("zero-digit"),

            // standard attributes
            standard: StandardNames::no_ns(source),
            // standard attributes on literal result elements
            xsl_standard: StandardNames::xsl(source, xsl_ns),
        }
    }

    pub(crate) fn sequence_constructor_name(
        &self,
        name: S::NameId,
    ) -> Option<SequenceConstructorName> {
        self.sequence_constructor_names.get(&name).copied()
    }

    pub(crate) fn declaration_name(&self, name: S::NameId) -> Option<DeclarationName> {
        self.declaration_names.get(&name).copied()
    }

    pub(crate) fn override_content_name(&self, name: S::NameId) -> Option<OverrideContentName> {
        self.override_content_names.get(&name).copied()
    }
}
