use crate::error;

pub fn one<'a, T>(mut iter: impl Iterator<Item = error::Result<T>> + 'a) -> error::Result<T> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(one?)
        } else {
            Err(error::Error::XPTY0004(
                "occurrence::one called with a sequence containing more than one item",
            ))
        }
    } else {
        Err(error::Error::XPTY0004(
            "occurrence::one called with an empty sequence",
        ))
    }
}

pub fn option<'a, T>(
    mut iter: impl Iterator<Item = error::Result<T>> + 'a,
) -> error::Result<Option<T>> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(Some(one?))
        } else {
            Err(error::Error::XPTY0004(
                "occurrence::option called with a sequence containing more than one item",
            ))
        }
    } else {
        Ok(None)
    }
}
