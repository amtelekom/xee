// https://www.w3.org/TR/xpath-functions-31/#array-functions

use ibig::IBig;

use xee_datasource::XmlNode;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::error;
use crate::function;
use crate::function::StaticFunctionDescription;
use crate::function::XmlDS;
use crate::interpreter;
use crate::interpreter::Interpreter;
use crate::sequence;
use crate::wrap_xpath_fn;

#[xpath_fn("map:merge($maps as map(*)*) as map(*)")]
fn merge1<N: XmlNode>(
    maps: impl Iterator<Item = error::Result<function::Map<N>>>,
) -> error::Result<function::Map<N>> {
    merge(
        maps,
        MergeOptions {
            duplicates: MergeDuplicates::UseFirst,
        },
    )
}

#[xpath_fn("map:merge($maps as map(*)*, $options as map(*)) as map(*)")]
fn merge2<S: XmlDS>(
    interpreter: &interpreter::Interpreter<S>,
    maps: impl Iterator<Item = error::Result<function::Map<S::Node>>>,
    options: function::Map<S::Node>,
) -> error::Result<function::Map<S::Node>> {
    let options = MergeOptions::from_map(&options, interpreter)?;
    merge(maps, options)
}

enum MergeDuplicates {
    // raise FOJS0003, if duplicate keys are encountered
    Reject,
    // if duplicate keys are present, the one from the earlier map takes precedence
    UseFirst,
    // if duplicate keys are present, the one from the later map takes precedence
    UseLast,
    // Implementation dependent on which duplicates take precedence
    UseAny,
    // duplicate values are concatenated into a sequence
    Combine,
}

impl MergeDuplicates {
    fn from_str(s: &str) -> error::Result<Self> {
        match s {
            "reject" => Ok(MergeDuplicates::Reject),
            "use-first" => Ok(MergeDuplicates::UseFirst),
            "use-last" => Ok(MergeDuplicates::UseLast),
            "use-any" => Ok(MergeDuplicates::UseAny),
            "combine" => Ok(MergeDuplicates::Combine),
            _ => Err(error::Error::FOJS0005),
        }
    }
}

struct MergeOptions {
    duplicates: MergeDuplicates,
}

impl MergeOptions {
    fn from_map<S: XmlDS>(
        map: &function::Map<S::Node>,
        interpreter: &interpreter::Interpreter<S>,
    ) -> error::Result<Self> {
        let key: atomic::Atomic = "duplicates".to_string().into();
        let duplicates = map.get(&key);
        if let Some(duplicates) = duplicates {
            let value = Self::duplicates_value(interpreter, duplicates)?;
            let duplicates = MergeDuplicates::from_str(&value)?;
            Ok(Self { duplicates })
        } else {
            // default
            Ok(Self {
                duplicates: MergeDuplicates::UseFirst,
            })
        }
    }

    fn duplicates_value<S: XmlDS>(
        interpreter: &interpreter::Interpreter<S>,
        duplicates: &sequence::Sequence<S::Node>,
    ) -> error::Result<String> {
        // we want a string type
        let sequence_type = ast::SequenceType::Item(ast::Item {
            occurrence: ast::Occurrence::One,
            item_type: ast::ItemType::AtomicOrUnionType(Xs::String),
        });
        // apply function conversion rules as specified by the option parameter
        // conventions
        let runnable = interpreter.runnable();
        let duplicates = duplicates
            .clone()
            .sequence_type_matching_function_conversion(
                &sequence_type,
                runnable.static_context(),
                interpreter.source(),
                &|function| runnable.program().function_info(function).signature(),
            )?;
        // take the first value, which should be a string
        let duplicates = duplicates.one()?;
        let atomic: atomic::Atomic = duplicates.to_atomic()?;
        atomic.to_string()
    }
}

fn merge<N: XmlNode>(
    maps: impl Iterator<Item = error::Result<function::Map<N>>>,
    options: MergeOptions,
) -> error::Result<function::Map<N>> {
    match options.duplicates {
        MergeDuplicates::Reject => function::Map::combine(maps, |_, _| Err(error::Error::FOJS0003)),
        MergeDuplicates::UseFirst => function::Map::combine(maps, |a, _| Ok(a)),
        MergeDuplicates::UseLast => function::Map::combine(maps, |_, b| Ok(b)),
        MergeDuplicates::UseAny => function::Map::combine(maps, |a, _| Ok(a)),
        MergeDuplicates::Combine => function::Map::combine(maps, |a, b| a.concat(b)),
    }
}

#[xpath_fn("map:size($map as map(*)) as xs:integer")]
fn size<N: XmlNode>(map: function::Map<N>) -> IBig {
    map.len().into()
}

#[xpath_fn("map:keys($map as map(*)) as xs:anyAtomicType*")]
fn keys<N: XmlNode>(map: function::Map<N>) -> sequence::Sequence<N> {
    map.keys().cloned().collect::<Vec<_>>().into()
}

#[xpath_fn("map:contains($map as map(*), $key as xs:anyAtomicType) as xs:boolean")]
fn contains<N: XmlNode>(map: function::Map<N>, key: atomic::Atomic) -> bool {
    map.get(&key).is_some()
}

#[xpath_fn("map:get($map as map(*), $key as xs:anyAtomicType) as item()*")]
fn get<N: XmlNode>(map: function::Map<N>, key: atomic::Atomic) -> sequence::Sequence<N> {
    map.get(&key).cloned().unwrap_or_default()
}

#[xpath_fn("map:find($input as item()*, $key as xs:anyAtomicType) as array(*)")]
fn find<N: XmlNode>(
    input: &sequence::Sequence<N>,
    key: atomic::Atomic,
) -> error::Result<function::Array<N>> {
    Ok(find_helper(input, atomic::MapKey::new(key.clone()).unwrap())?.into())
}

fn find_helper<N: XmlNode>(
    input: &sequence::Sequence<N>,
    key: atomic::MapKey,
) -> error::Result<Vec<sequence::Sequence<N>>> {
    let mut result: Vec<sequence::Sequence<N>> = Vec::new();
    for item in input.iter() {
        if let sequence::Item::Function(function) = item {
            match function {
                function::Function::Array(array) => {
                    for entry in array.iter() {
                        let found = find_helper(entry, key.clone())?;
                        result.extend(found)
                    }
                }
                function::Function::Map(map) => {
                    for (k, v) in map.map_key_entries() {
                        if k == &key {
                            result.push(v.clone());
                        }
                        let found = find_helper(v, key.clone())?;
                        result.extend(found)
                    }
                }
                _ => {}
            }
        }
    }
    Ok(result)
}

#[xpath_fn("map:put($map as map(*), $key as xs:anyAtomicType, $value as item()*) as map(*)")]
fn put<N: XmlNode>(
    map: function::Map<N>,
    key: atomic::Atomic,
    value: &sequence::Sequence<N>,
) -> error::Result<function::Map<N>> {
    map.put(key, value)
}

#[xpath_fn("map:entry($key as xs:anyAtomicType, $value as item()*) as map(*)")]
fn entry<N: XmlNode>(key: atomic::Atomic, value: &sequence::Sequence<N>) -> function::Map<N> {
    function::Map::new(vec![(key, value.clone())]).unwrap()
}

#[xpath_fn("map:remove($map as map(*), $keys as xs:anyAtomicType*) as map(*)")]
fn remove<N: XmlNode>(
    map: function::Map<N>,
    keys: impl Iterator<Item = error::Result<atomic::Atomic>>,
) -> error::Result<function::Map<N>> {
    map.remove_keys(keys)
}

#[xpath_fn("map:for-each($map as map(*), $action as function(xs:anyAtomicType, item()*) as item()*) as item()*")]
fn for_each<S: XmlDS>(
    interpreter: &mut Interpreter<S>,
    map: function::Map<S::Node>,
    action: sequence::Item<S::Node>,
) -> error::Result<sequence::Sequence<S::Node>> {
    let function = action.to_function()?;
    let mut result: Vec<sequence::Item<S::Node>> = Vec::with_capacity(map.len());
    for (key, value) in map.entries() {
        let r = interpreter
            .call_function_with_arguments(&function, &[key.clone().into(), value.clone()])?;
        for item in r.iter() {
            result.push(item.clone());
        }
    }
    Ok(result.into())
}

pub(crate) fn static_function_descriptions<S: XmlDS>() -> Vec<StaticFunctionDescription<S>> {
    vec![
        wrap_xpath_fn!(merge1, S),
        wrap_xpath_fn!(merge2, S),
        wrap_xpath_fn!(find, S),
        wrap_xpath_fn!(size, S),
        wrap_xpath_fn!(keys, S),
        wrap_xpath_fn!(contains, S),
        wrap_xpath_fn!(get, S),
        wrap_xpath_fn!(put, S),
        wrap_xpath_fn!(entry, S),
        wrap_xpath_fn!(remove, S),
        wrap_xpath_fn!(for_each, S),
    ]
}
