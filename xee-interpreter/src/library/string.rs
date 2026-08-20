// https://www.w3.org/TR/xpath-functions-31/#string-functions

use std::cmp::Ordering;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use ibig::IBig;
use icu::normalizer::{ComposingNormalizer, DecomposingNormalizer};

use crate::function::XmlDS;
use regexml::{AnalyzeEntry, MatchEntry};
use xee_datasource::{XmlMutAttribAccess, XmlMutNamespaceAccess, XmlNodeWrapper};
use xee_name::{Name, FN_NAMESPACE};
use xee_schema_type::Xs;
use xee_xpath_macros::xpath_fn;
use xee_xpath_type::ast;

use crate::context::DynamicContext;
use crate::function::{self, StaticFunctionDescription};
use crate::interpreter::Interpreter;
use crate::string::Collation;
use crate::{atomic, error, interpreter, occurrence, sequence, wrap_xpath_fn};

// we don't accept concat() invocations with an arity greater than this
const MAX_CONCAT_ARITY: usize = 99;

#[xpath_fn("fn:codepoints-to-string($arg as xs:integer*) as xs:string")]
fn codepoints_to_string(arg: impl Iterator<Item = error::Result<IBig>>) -> error::Result<String> {
    arg.map(|c| {
        let c = c?;
        let c: u32 = c.try_into().map_err(|_| error::Error::FOCH0001)?;
        let c = char::from_u32(c).ok_or(error::Error::FOCH0001)?;
        if is_valid_xml_char(c) {
            Ok(c)
        } else {
            Err(error::Error::FOCH0001)
        }
    })
    .collect::<error::Result<String>>()
}

fn is_valid_xml_char(c: char) -> bool {
    // Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
    c == '\t'
        || c == '\n'
        || c == '\r'
        || ('\u{20}'..='\u{D7FF}').contains(&c)
        || ('\u{E000}'..='\u{FFFD}').contains(&c)
        || ('\u{10000}'..='\u{10FFFF}').contains(&c)
}

#[xpath_fn("fn:string-to-codepoints($arg as xs:string?) as xs:integer*")]
fn string_to_codepoints(arg: Option<&str>) -> error::Result<Vec<IBig>> {
    if let Some(arg) = arg {
        Ok(arg.chars().map(|c| c as u32).map(IBig::from).collect())
    } else {
        // empty sequence
        Ok(Vec::new())
    }
}

// https://www.w3.org/TR/xpath-functions-31/#string-functions
#[xpath_fn(
    "fn:compare($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:integer?",
    collation
)]
fn compare<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<Option<IBig>> {
    if let (Some(arg1), Some(arg2)) = (arg1, arg2) {
        let collation = context
            .static_context()
            .resolve_collation_str(Some(collation))?;
        Ok(Some(
            match collation.compare(arg1, arg2) {
                Ordering::Equal => 0,
                Ordering::Less => -1,
                Ordering::Greater => 1,
            }
            .into(),
        ))
    } else {
        Ok(None)
    }
}

#[xpath_fn(
    "fn:codepoint-equal($comparand1 as xs:string?, $comparand2 as xs:string?) as xs:boolean?"
)]
fn codepoint_equal(comparand1: Option<&str>, comparand2: Option<&str>) -> Option<bool> {
    if let (Some(comparand1), Some(comparand2)) = (comparand1, comparand2) {
        Some(comparand1 == comparand2)
    } else {
        None
    }
}

#[xpath_fn(
    "fn:contains-token($input as xs:string*, $token as xs:string, $collation as xs:string) as xs:boolean",
    collation
)]
fn contains_token<S: XmlDS>(
    context: &DynamicContext<S>,
    input: impl Iterator<Item = error::Result<String>>,
    token: &str,
    collation: &str,
) -> error::Result<bool> {
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    let token = token.trim();
    for s in input {
        let s = s?;
        // if any token in s, tokenized, is token, then we return true
        if s.split_ascii_whitespace()
            .any(|t| collation.compare(t, token).is_eq())
        {
            return Ok(true);
        }
    }
    Ok(false)
}

// concat cannot be written using the macro system, as it
// takes an arbitrary amount of arguments. This is the only
// function that does this. We're going to define a general
// concat function and then register it for a sufficient amount
// of arities
fn concat<S: XmlDS>(
    _context: &DynamicContext<S>,
    interpreter: &mut interpreter::Interpreter<S>,
    arguments: &[sequence::Sequence<S::Node>],
) -> error::Result<sequence::Sequence<S::Node>> {
    debug_assert!(arguments.len() >= 2);

    let strings = arguments
        .iter()
        .map(|argument| {
            let atomic = occurrence::option(argument.atomized(interpreter.source()))?;
            if let Some(atomic) = atomic {
                Ok(atomic.string_value())
            } else {
                Ok("".to_string())
            }
        })
        .collect::<error::Result<Vec<String>>>()?;
    Ok(strings.concat().into())
}

#[xpath_fn("fn:string-join($arg1 as xs:anyAtomicType*) as xs:string")]
fn string_join(arg1: impl Iterator<Item = error::Result<atomic::Atomic>>) -> error::Result<String> {
    let arg1 = arg1
        .map(|a| Ok(a?.string_value()))
        .collect::<error::Result<Vec<String>>>()?;
    Ok(arg1.concat())
}

#[xpath_fn("fn:string-join($arg1 as xs:anyAtomicType*, $arg2 as xs:string) as xs:string")]
fn string_join_sep(
    arg1: impl Iterator<Item = error::Result<atomic::Atomic>>,
    arg2: &str,
) -> error::Result<String> {
    let arg1 = arg1
        .map(|a| Ok(a?.string_value()))
        .collect::<error::Result<Vec<String>>>()?;
    Ok(arg1.join(arg2))
}

#[xpath_fn("fn:substring($sourceString as xs:string?, $start as xs:double) as xs:string")]
fn substring2(source_string: Option<&str>, start: f64) -> String {
    if let Some(source_string) = source_string {
        substring_with_length(source_string, start, f64::INFINITY)
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:substring($sourceString as xs:string?, $start as xs:double, $length as xs:double) as xs:string")]
fn substring3(source_string: Option<&str>, start: f64, length: f64) -> String {
    if let Some(source_string) = source_string {
        substring_with_length(source_string, start, length)
    } else {
        "".to_string()
    }
}

fn substring_with_length(source_string: &str, start: f64, length: f64) -> String {
    // we deliberately do the calculations with floats as long as possible
    // to handle infinities and such, as those are part of the spec, as well
    // as avoid overflows.
    if source_string.is_empty() {
        return "".to_string();
    }
    if start.is_nan() || length.is_nan() {
        return "".to_string();
    }
    let start = start.round();
    let length = length.round();

    // we calculate the end point
    let end = start + length;
    // if this results in a NaN we're done
    if end.is_nan() {
        return "".to_string();
    }
    // we say the start should not be less than 1
    let start = start.round().max(1f64);
    // now we say the end should not more than the total length of the string
    // the end position is one beyond the end of the string, due to the starting
    // at 1.
    let end = end.min((source_string.len() + 1) as f64);

    // now turn into integers and substract 1 to get
    let length: usize = (end - start) as usize;
    let start: usize = start as usize - 1;

    source_string
        .chars()
        .skip(start)
        // take until end position is reached
        .take(length)
        .collect::<String>()
}

#[xpath_fn("fn:string-length($arg as xs:string?) as xs:integer", context_first)]
fn string_length(arg: Option<&str>) -> IBig {
    if let Some(arg) = arg {
        // TODO: what about overflow? not a very realistic situation
        arg.chars().count().into()
    } else {
        0.into()
    }
}

#[xpath_fn("fn:normalize-space($arg as xs:string?) as xs:string", context_first)]
fn normalize_space(arg: Option<&str>) -> String {
    if let Some(arg) = arg {
        arg.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:normalize-unicode($arg as xs:string?) as xs:string")]
fn normalize_unicode1(arg: Option<&str>) -> error::Result<String> {
    normalize_unicode(arg, "NFC")
}

#[xpath_fn(
    "fn:normalize-unicode($arg as xs:string?, $normalizationForm as xs:string) as xs:string"
)]
fn normalize_unicode2(arg: Option<&str>, normalization_form: &str) -> error::Result<String> {
    normalize_unicode(arg, normalization_form)
}

fn normalize_unicode(arg: Option<&str>, normalization_form: &str) -> error::Result<String> {
    if let Some(arg) = arg {
        let normalization_form = normalization_form
            .split_ascii_whitespace()
            .collect::<String>()
            .to_uppercase();
        if normalization_form.is_empty() {
            return Ok(arg.to_string());
        }

        match normalization_form.as_ref() {
            "NFC" => {
                let normalizer = ComposingNormalizer::new_nfc();
                Ok(normalizer.normalize(arg))
            }
            "NFD" => {
                let normalizer = DecomposingNormalizer::new_nfd();
                Ok(normalizer.normalize(arg))
            }
            "NFKC" => {
                let normalizer = ComposingNormalizer::new_nfkc();
                Ok(normalizer.normalize(arg))
            }
            "NFKD" => {
                let normalizer = DecomposingNormalizer::new_nfkd();
                Ok(normalizer.normalize(arg))
            }
            // TODO: FULLY-NORMALIZED
            _ => Err(error::Error::FOCH0003),
        }
    } else {
        Ok("".to_string())
    }
}

// TODO: fn:normalize-unicode

#[xpath_fn("fn:upper-case($arg as xs:string?) as xs:string")]
fn upper_case(arg: Option<&str>) -> String {
    if let Some(arg) = arg {
        arg.to_uppercase()
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:lower-case($arg as xs:string?) as xs:string")]
fn lower_case(arg: Option<&str>) -> String {
    if let Some(arg) = arg {
        arg.to_lowercase()
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:tokenize($input as xs:string?) as xs:string*")]
fn tokenize1(input: Option<&str>) -> error::Result<Vec<String>> {
    if let Some(input) = input {
        Ok(input
            .split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect())
    } else {
        Ok(Vec::new())
    }
}

#[xpath_fn("fn:translate($arg as xs:string?, $mapString as xs:string, $transString as xs:string) as xs:string")]
fn translate(arg: Option<&str>, map_string: &str, trans_string: &str) -> String {
    if let Some(arg) = arg {
        let mut map = HashMap::new();
        let mut ignore_set = HashSet::new();
        let map_string_chars = map_string.chars();
        let mut trans_string_chars = trans_string.chars();
        for char in map_string_chars {
            let trans = trans_string_chars.next();
            if let Some(trans) = trans {
                map.insert(char, trans);
            } else {
                ignore_set.insert(char);
            }
        }
        let mut o = String::with_capacity(arg.len());
        for c in arg.chars() {
            match map.get(&c) {
                Some(rep) => o.push(*rep),
                None => {
                    if !ignore_set.contains(&c) {
                        o.push(c)
                    }
                }
            }
        }
        o
    } else {
        "".to_string()
    }
}

#[xpath_fn(
    "fn:contains($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean",
    collation
)]
fn contains<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<bool> {
    let arg1 = arg1.unwrap_or("");
    let arg2 = arg2.unwrap_or("");
    if arg2.is_empty() {
        return Ok(true);
    }
    if arg1.is_empty() {
        return Ok(false);
    }
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    match collation.as_ref() {
        Collation::CodePoint => Ok(arg1.contains(arg2)),
        Collation::HtmlAscii => {
            let arg1 = arg1.to_ascii_lowercase();
            let arg2 = arg2.to_ascii_lowercase();
            Ok(arg1.contains(&arg2))
        }
        // for now, icu4x does not yet support collation units (actually named collation elements)
        // https://github.com/unicode-org/icu4x/discussions/3981
        Collation::Uca(_) => Err(error::Error::FOCH0004),
    }
}

#[xpath_fn("fn:starts-with($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean", collation)]
fn starts_with<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<bool> {
    let arg1 = arg1.unwrap_or("");
    let arg2 = arg2.unwrap_or("");
    if arg2.is_empty() {
        return Ok(true);
    }
    if arg1.is_empty() {
        return Ok(false);
    }
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    match collation.as_ref() {
        Collation::CodePoint => Ok(arg1.starts_with(arg2)),
        Collation::HtmlAscii => {
            let arg1 = arg1.to_lowercase();
            let arg2 = arg2.to_lowercase();
            Ok(arg1.starts_with(&arg2))
        }
        // for now, icu4x does not yet support collation units (actually named collation elements)
        // https://github.com/unicode-org/icu4x/discussions/3981
        Collation::Uca(_) => Err(error::Error::FOCH0004),
    }
}

#[xpath_fn(
    "fn:ends-with($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:boolean",
    collation
)]
fn ends_with<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<bool> {
    let arg1 = arg1.unwrap_or("");
    let arg2 = arg2.unwrap_or("");
    if arg2.is_empty() {
        return Ok(true);
    }
    if arg1.is_empty() {
        return Ok(false);
    }
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    match collation.as_ref() {
        Collation::CodePoint => Ok(arg1.ends_with(arg2)),
        Collation::HtmlAscii => {
            let arg1 = arg1.to_lowercase();
            let arg2 = arg2.to_lowercase();
            Ok(arg1.ends_with(&arg2))
        }
        // for now, icu4x does not yet support collation units (actually named collation elements)
        // https://github.com/unicode-org/icu4x/discussions/3981
        Collation::Uca(_) => Err(error::Error::FOCH0004),
    }
}

#[xpath_fn("fn:substring-before($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:string", collation)]
fn substring_before<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<String> {
    let arg1 = arg1.unwrap_or("");
    let arg2 = arg2.unwrap_or("");
    if arg2.is_empty() {
        return Ok("".to_string());
    }
    // find substring in arg1 that comes before arg2
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    match collation.as_ref() {
        Collation::CodePoint => {
            let idx = arg1.find(arg2).unwrap_or(0);
            Ok(arg1[..idx].to_string())
        }
        Collation::HtmlAscii => {
            let arg1_l = arg1.to_lowercase();
            let arg2_l = arg2.to_lowercase();
            let idx = arg1_l.find(&arg2_l).unwrap_or(0);
            Ok(arg1[..idx].to_string())
        }
        // for now, icu4x does not yet support collation units (actually named collation elements)
        // https://github.com/unicode-org/icu4x/discussions/3981
        Collation::Uca(_) => Err(error::Error::FOCH0004),
    }
}

#[xpath_fn("fn:substring-after($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:string", collation)]
fn substring_after<S: XmlDS>(
    context: &DynamicContext<S>,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<String> {
    let arg1 = arg1.unwrap_or("");
    let arg2 = arg2.unwrap_or("");
    if arg2.is_empty() {
        return Ok(arg1.to_string());
    }
    // find substring in arg1 that comes before arg2
    let collation = context
        .static_context()
        .resolve_collation_str(Some(collation))?;
    match collation.as_ref() {
        Collation::CodePoint => {
            if let Some(idx) = arg1.find(arg2) {
                Ok(arg1[(idx + arg2.len())..].to_string())
            } else {
                Ok("".to_string())
            }
        }
        Collation::HtmlAscii => {
            let arg1_l = arg1.to_lowercase();
            let arg2_l = arg2.to_lowercase();
            if let Some(idx) = arg1_l.find(&arg2_l) {
                Ok(arg1[(idx + arg2.len())..].to_string())
            } else {
                Ok("".to_string())
            }
        }
        // for now, icu4x does not yet support collation units (actually named collation elements)
        // https://github.com/unicode-org/icu4x/discussions/3981
        Collation::Uca(_) => Err(error::Error::FOCH0004),
    }
}

#[xpath_fn(
    "fn:matches($input as xs:string?, $pattern as xs:string, $flags as xs:string) as xs:boolean"
)]
fn matches3<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<bool> {
    matches(interpreter, input, pattern, flags)
}

#[xpath_fn("fn:matches($input as xs:string?, $pattern as xs:string) as xs:boolean")]
fn matches2<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
) -> error::Result<bool> {
    matches(interpreter, input, pattern, "")
}

fn matches<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<bool> {
    let regex = interpreter.regex(pattern, flags)?;
    let input = input.unwrap_or("");
    Ok(regex.is_match(input))
}

#[xpath_fn("fn:replace($input as xs:string?, $pattern as xs:string, $replacement as xs:string, $flags as xs:string) as xs:string")]
fn replace4<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    replacement: &str,
    flags: &str,
) -> error::Result<String> {
    replace(interpreter, input, pattern, replacement, flags)
}

#[xpath_fn("fn:replace($input as xs:string?, $pattern as xs:string, $replacement as xs:string) as xs:string")]
fn replace3<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    replacement: &str,
) -> error::Result<String> {
    replace(interpreter, input, pattern, replacement, "")
}

fn replace<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    replacement: &str,
    flags: &str,
) -> error::Result<String> {
    let regex = interpreter.regex(pattern, flags)?;
    let input = input.unwrap_or("");
    Ok(regex.replace_all(input, replacement)?)
}

#[xpath_fn(
    "fn:tokenize($input as xs:string?, $pattern as xs:string, $flags as xs:string) as xs:string*"
)]
fn tokenize3<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<Vec<String>> {
    tokenize(interpreter, input, pattern, flags)
}

#[xpath_fn("fn:tokenize($input as xs:string?, $pattern as xs:string) as xs:string*")]
fn tokenize2<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
) -> error::Result<Vec<String>> {
    tokenize(interpreter, input, pattern, "")
}

fn tokenize<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<Vec<String>> {
    let regex = interpreter.regex(pattern, flags)?;
    let input = input.unwrap_or("");
    Ok(regex.tokenize(input)?.collect::<Vec<_>>())
}

#[xpath_fn("fn:analyze-string($input as xs:string?, $pattern as xs:string) as element()")]
fn analyze_string2<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
) -> error::Result<sequence::Sequence<S::Node>> {
    analyze_string(interpreter, input, pattern, "")
}

#[xpath_fn(
    "fn:analyze-string($input as xs:string?, $pattern as xs:string, $flags as xs:string) as element()"
)]
fn analyze_string3<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<sequence::Sequence<S::Node>> {
    analyze_string(interpreter, input, pattern, flags)
}

struct AnalyzeStringNames<S: XmlDS> {
    fn_prefix: S::PrefixId,
    fn_namespace: S::NamespaceId,
    analyze_string_result: S::NameId,
    match_: S::NameId,
    non_match: S::NameId,
    group_name: S::NameId,
    group_nr: S::NameId,
}

impl<S: XmlDS> AnalyzeStringNames<S> {
    fn new(source: &mut S) -> Self {
        let fn_namespace = source.add_namespace(FN_NAMESPACE);
        Self {
            fn_prefix: source.add_prefix("fn"),
            fn_namespace,
            analyze_string_result: source.add_name_ns("analyze-string-result", fn_namespace),
            match_: source.add_name_ns("match", fn_namespace),
            non_match: source.add_name_ns("non-match", fn_namespace),
            group_name: source.add_name_ns("group", fn_namespace),
            group_nr: source.add_name("nr"),
        }
    }
}

fn analyze_string<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    input: Option<&str>,
    pattern: &str,
    flags: &str,
) -> error::Result<sequence::Sequence<S::Node>> {
    let regex = interpreter.regex(pattern, flags)?;
    let input = input.unwrap_or("");
    let analyze_results = regex.analyze(input)?;

    let source = interpreter.state.source_mut();
    // TODO: do this somewhere on startup time so we don't need to do it for each
    // call
    let analyze_string_names = AnalyzeStringNames::new(source);

    let analyze_string_result = source.new_element(analyze_string_names.analyze_string_result);
    let mut namespaces = source.namespaces_mut(analyze_string_result);
    namespaces.insert(
        analyze_string_names.fn_prefix,
        analyze_string_names.fn_namespace,
    );
    drop(namespaces);
    for entry in analyze_results {
        let child = match entry {
            AnalyzeEntry::Match(match_entries) => {
                let match_node = source.new_element(analyze_string_names.match_);
                serialize_match_entries(source, &analyze_string_names, match_node, &match_entries);
                match_node
            }
            AnalyzeEntry::NonMatch(s) => {
                let non_match_node = source.new_element(analyze_string_names.non_match);
                let text = source.new_text(&s);
                source.append(non_match_node, text).unwrap();
                non_match_node
            }
        };
        source.append(analyze_string_result, child).unwrap();
    }
    let item: sequence::Item<_> = XmlNodeWrapper(analyze_string_result).into();
    let sequence: sequence::Sequence<_> = item.into();
    Ok(sequence)
}

fn serialize_match_entries<S: XmlDS>(
    source: &mut S,
    analyze_string_names: &AnalyzeStringNames<S>,
    node: S::Node,
    match_entries: &[MatchEntry],
) {
    for entry in match_entries {
        let child = match entry {
            MatchEntry::String(s) => source.new_text(s),
            MatchEntry::Group { nr, value } => {
                let group = source.new_element(analyze_string_names.group_name);
                let mut attributes = source.attributes_mut(group);
                attributes.insert(analyze_string_names.group_nr, nr.to_string());
                drop(attributes);
                serialize_match_entries(source, analyze_string_names, group, value);
                group
            }
        };
        source.append(node, child).unwrap();
    }
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    let mut r = vec![
        wrap_xpath_fn!(codepoints_to_string, S),
        wrap_xpath_fn!(string_to_codepoints, S),
        wrap_xpath_fn!(compare, S),
        wrap_xpath_fn!(codepoint_equal, S),
        wrap_xpath_fn!(contains_token, S),
        wrap_xpath_fn!(string_join, S),
        wrap_xpath_fn!(string_join_sep, S),
        wrap_xpath_fn!(substring2, S),
        wrap_xpath_fn!(substring3, S),
        wrap_xpath_fn!(string_length, S),
        wrap_xpath_fn!(normalize_space, S),
        wrap_xpath_fn!(normalize_unicode1, S),
        wrap_xpath_fn!(normalize_unicode2, S),
        wrap_xpath_fn!(upper_case, S),
        wrap_xpath_fn!(lower_case, S),
        wrap_xpath_fn!(translate, S),
        wrap_xpath_fn!(contains, S),
        wrap_xpath_fn!(starts_with, S),
        wrap_xpath_fn!(ends_with, S),
        wrap_xpath_fn!(substring_before, S),
        wrap_xpath_fn!(substring_after, S),
        wrap_xpath_fn!(tokenize1, S),
        wrap_xpath_fn!(matches2, S),
        wrap_xpath_fn!(matches3, S),
        wrap_xpath_fn!(replace3, S),
        wrap_xpath_fn!(replace4, S),
        wrap_xpath_fn!(tokenize3, S),
        wrap_xpath_fn!(tokenize2, S),
        wrap_xpath_fn!(analyze_string2, S),
        wrap_xpath_fn!(analyze_string3, S),
    ];
    // register concat for a variety of arities
    // the spec leaves the amount of arguments indefinite
    // https://www.w3.org/TR/xpath-functions-31/#func-concat
    let arg_type = ast::SequenceType::Item(ast::Item {
        occurrence: ast::Occurrence::Option,
        item_type: ast::ItemType::AtomicOrUnionType(Xs::AnyAtomicType),
    });
    let string_type = ast::SequenceType::Item(ast::Item {
        occurrence: ast::Occurrence::One,
        item_type: ast::ItemType::AtomicOrUnionType(Xs::String),
    });
    let name = Name::new(
        "concat".to_string(),
        FN_NAMESPACE.to_string(),
        String::new(),
    );

    for arity in 2..=MAX_CONCAT_ARITY {
        let signature = function::Signature::new(
            vec![Some(arg_type.clone()); arity],
            Some(string_type.clone()),
        );
        r.push(StaticFunctionDescription::<S> {
            name: name.clone(),
            signature,
            function_kind: None,
            func: concat,
        });
    }
    r
}
