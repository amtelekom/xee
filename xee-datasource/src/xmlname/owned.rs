// Note: This is a bit of an exception, not adding a Trait but instead either using the Xot struct as a newtype
// if linking Xot, or a copy of the same struct if not, since it would 'infect' a huge swath of structs with
// rather complicated generic params throughout Xee if dependent on the XmlDS - for a struct storing 3 Strings...
// We do the newtype if linking Xot since some apis are not public in Xot, so if we didn't we'd need to copy around Strings to create AsRefs...

// The following is based on the implementation from XOT and slightly modified to work with the generic traits:
// Copyright (c) 2022 Martijn Faassen & Paligo

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[cfg(feature = "xot")]
use crate::XmlDS;
use crate::{error::Error, xmlname::NameStrInfo};

// make 'Serialize' point to serde if feature is enabled, empty marker trait otherwise:
// #[cfg(feature = "serde")]
// use serde::Serialize;
// #[cfg(not(feature = "serde"))]
// pub trait Serialize {}
// #[cfg(not(feature = "serde"))]
// impl<S> Serialize for S {}

/// An owned name stores the name, namespace and prefix as owned strings.
///
/// An owned name is handy when you don't want to depend on Xot, or for
/// debugging.
///
/// It cannot be used to create elements directly, but you can turn this into a
/// [`CreateName`] using [`OwnedName::to_create`] and a [`RefName`] using
/// [`OwnedName::to_ref`].
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// If you enable the `serde` feature it can be serialized and deserialized.
#[cfg(feature = "xot")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnedName(pub(crate) xot::xmlname::OwnedName);

#[cfg(feature = "xot")]
impl OwnedName {
    /// Create a new owned name.
    #[inline]
    pub fn new(local_name: String, namespace: String, prefix: String) -> Self {
        Self(::xot::xmlname::OwnedName::new(
            local_name, namespace, prefix,
        ))
    }

    /// Create a name without a namespace
    pub fn name(local_name: &str) -> Self {
        Self(::xot::xmlname::OwnedName::name(local_name))
    }

    /// Create a new name in a namespace, look up prefix information.
    pub fn namespaced(
        local_name: String,
        namespace: String,
        prefix_lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self(::xot::xmlname::OwnedName::namespaced(
            local_name,
            namespace,
            prefix_lookup,
        )?))
    }

    /// Create a new owned name from a prefix and a name.
    pub fn prefixed(
        prefix: &str,
        local_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self(::xot::xmlname::OwnedName::prefixed(
            prefix,
            local_name,
            lookup_namespace,
        )?))
    }

    /// Given a fullname (with potentially a prefix), construct an XmlNameOwned
    ///
    /// This requires a function that can look up the namespace for a prefix.
    pub fn parse_full_name(
        full_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self(::xot::xmlname::OwnedName::parse_full_name(
            full_name,
            lookup_namespace,
        )?))
    }

    /// Convert this name into a name adding a * suffix.
    ///
    /// This can be useful to help generate unique names.
    pub fn with_suffix(self) -> Self {
        Self(self.0.with_suffix())
    }

    /// Convert this name into a name that's in a particular namespace.
    ///
    /// This only changes the namespace if there is an empty prefix and the
    /// namespace is not set (the empty string).
    pub fn with_default_namespace(self, namespace: &str) -> Self {
        Self(self.0.with_default_namespace(namespace))
    }

    /// Name is in a namespace but without a prefix, so it's in a default namespace.
    pub fn in_default_namespace(&self) -> bool {
        self.0.in_default_namespace()
    }

    /// Note: This is only implemented for the Xot backend, and can't be used in generic contexts
    pub fn to_ref<'a>(&self, xot: &'a mut xot::Xot) -> <xot::Xot as XmlDS>::RefName<'a> {
        self.0.to_ref(xot)
    }
}

#[cfg(feature = "xot")]
impl NameStrInfo for OwnedName {
    fn local_name(&self) -> &str {
        self.0.local_name()
    }

    fn namespace(&self) -> &str {
        self.0.namespace()
    }

    fn prefix(&self) -> &str {
        self.0.prefix()
    }
}

#[cfg(feature = "xot")]
impl std::fmt::Debug for OwnedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //Hide newtype in debug prints:
        self.0.fmt(f)
    }
}

#[cfg(all(feature = "xot", feature = "serde"))]
impl serde::Serialize for OwnedName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        //Hide newtype in serializations:
        self.0.serialize(serializer)
    }
}

#[cfg(not(feature = "xot"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OwnedName {
    // fields are postfixed _str so they don't conflict with the methods in
    // NameStrInfo
    local_name_str: String,
    // the empty namespace uri means no namespace
    namespace_str: String,
    // the empty prefix means no prefix.
    prefix_str: String,
}

#[cfg(not(feature = "xot"))]
impl OwnedName {}
