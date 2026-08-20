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

use std::borrow::Cow;

use crate::{XmlDS, xmlname::OwnedName};

/// Name string information for an xml name.
pub trait NameStrInfo {
    /// Access the local name as a string reference
    fn local_name(&self) -> &str;

    /// Get the namespace uri as a str reference.
    ///
    /// If there is no namespace, this is the empty string.
    fn namespace(&self) -> &str;

    /// Access the prefix as a string
    fn prefix(&self) -> &str;

    /// Access the full name as a string
    fn full_name(&self) -> Cow<'_, str> {
        let prefix = self.prefix();
        if !prefix.is_empty() {
            Cow::Owned(format!("{}:{}", prefix, self.local_name()))
        } else {
            Cow::Borrowed(self.local_name())
        }
    }
}

/// The most complete way to access name information, backed by Xot. This is a
/// reference and cannot be created directly.
///
/// You can create one by using [`OwnedName`], or by using the
/// [`Xot::name_ref`] and [`Xot::node_name_ref`] methods.
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// It can also be used directly to create new elements and attributes, instead
/// of a [`crate::NameId`].
pub trait RefName<'a, S: XmlDS>: NameStrInfo + std::hash::Hash + PartialEq + Eq + Sized {
    /// Get the Xot name id.
    fn name_id(&self) -> S::NameId;

    /// Get the Xot namespace id.
    fn namespace_id(&self) -> S::NamespaceId;

    /// Get the Xot prefix id.
    fn prefix_id(&self) -> S::PrefixId;

    /// Create a new [`OwnedName`] from this reference.
    ///
    /// Normally you shouldn't have to do this because you can already access
    /// the name string information on this reference using [`NameStrInfo`].
    fn to_owned(&self) -> OwnedName {
        OwnedName::new(
            self.local_name().to_string(),
            self.namespace().to_string(),
            self.prefix().to_string(),
        )
    }

    /// Has a namespace but no prefix, so it's in a `xmlns` namespace.
    fn has_unprefixed_namespace(&self) -> bool;
}
