// the stack::Value is either a sequence or absent

use xee_datasource::XmlNode;

use crate::error;

use crate::sequence;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<N: XmlNode> {
    Absent,
    Sequence(sequence::Sequence<N>),
}

impl<N: XmlNode> Value<N> {
    pub(crate) fn is_absent(&self) -> bool {
        matches!(self, Value::Absent)
    }
}

impl<N: XmlNode> TryFrom<Value<N>> for sequence::Sequence<N> {
    type Error = error::Error;

    fn try_from(value: Value<N>) -> Result<Self, Self::Error> {
        match value {
            Value::Absent => Err(error::Error::XPDY0002),
            Value::Sequence(sequence) => Ok(sequence),
        }
    }
}

impl<N: XmlNode> TryFrom<&Value<N>> for sequence::Sequence<N> {
    type Error = error::Error;

    fn try_from(value: &Value<N>) -> Result<Self, Self::Error> {
        match value {
            Value::Absent => Err(error::Error::XPDY0002),
            Value::Sequence(sequence) => Ok(sequence.clone()),
        }
    }
}

impl<T, N: XmlNode> From<T> for Value<N>
where
    T: Into<sequence::Sequence<N>>,
{
    fn from(t: T) -> Self {
        let sequence: sequence::Sequence<N> = t.into();
        Value::Sequence(sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // a static assertion to ensure that Value never grows in size
    #[cfg(target_arch = "x86_64")]
    static_assertions::assert_eq_size!(Value<xot::Node>, [u8; 24]);
}
