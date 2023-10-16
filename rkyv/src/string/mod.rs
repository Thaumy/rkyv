//! Archived versions of string types.

pub mod repr;

use crate::SerializeUnsized;
use core::{
    borrow::Borrow,
    cmp, fmt, hash,
    ops::{
        Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
        RangeToInclusive,
    },
    pin::Pin,
    str,
};
use repr::{ArchivedStringRepr, INLINE_CAPACITY};

/// An archived [`String`].
///
/// This has inline and out-of-line representations. Short strings will use the available space
/// inside the structure to store the string, and long strings will store a
/// [`RelPtr`](crate::RelPtr) to a `str` instead.
#[repr(transparent)]
#[cfg_attr(
    feature = "bytecheck",
    derive(bytecheck::CheckBytes),
    check_bytes(
        bounds(
            __C: crate::validation::ArchiveContext<__E>,
            __E: bytecheck::rancor::Error,
        ),
        verify = verify::verify,
    ),
)]
pub struct ArchivedString {
    repr: repr::ArchivedStringRepr,
}

impl ArchivedString {
    /// Extracts a string slice containing the entire `ArchivedString`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }

    /// Extracts a pinned mutable string slice containing the entire `ArchivedString`.
    #[inline]
    pub fn pin_mut_str(self: Pin<&mut Self>) -> Pin<&mut str> {
        unsafe { self.map_unchecked_mut(|s| s.repr.as_mut_str()) }
    }

    /// Resolves an archived string from a given `str`.
    ///
    /// # Safety
    ///
    /// - `pos` must be the position of `out` within the archive
    /// - `resolver` must be the result of serializing `value`
    #[inline]
    pub unsafe fn resolve_from_str(
        value: &str,
        pos: usize,
        resolver: StringResolver,
        out: *mut Self,
    ) {
        if value.len() <= repr::INLINE_CAPACITY {
            ArchivedStringRepr::emplace_inline(value, out.cast());
        } else {
            ArchivedStringRepr::emplace_out_of_line(
                value,
                pos,
                resolver.pos,
                out.cast(),
            );
        }
    }

    /// Serializes an archived string from a given `str`.
    #[inline]
    pub fn serialize_from_str<S: ?Sized, E>(
        value: &str,
        serializer: &mut S,
    ) -> Result<StringResolver, E>
    where
        str: SerializeUnsized<S, E>,
    {
        if value.len() <= INLINE_CAPACITY {
            Ok(StringResolver { pos: 0 })
        } else {
            Ok(StringResolver {
                pos: value.serialize_unsized(serializer)?,
            })
        }
    }
}

impl AsRef<str> for ArchivedString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for ArchivedString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for ArchivedString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl Deref for ArchivedString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for ArchivedString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl Eq for ArchivedString {}

impl hash::Hash for ArchivedString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

macro_rules! impl_index {
    ($index:ty) => {
        impl Index<$index> for ArchivedString {
            type Output = str;

            #[inline]
            fn index(&self, index: $index) -> &Self::Output {
                self.as_str().index(index)
            }
        }
    };
}

impl_index!(Range<usize>);
impl_index!(RangeFrom<usize>);
impl_index!(RangeFull);
impl_index!(RangeInclusive<usize>);
impl_index!(RangeTo<usize>);
impl_index!(RangeToInclusive<usize>);

impl Ord for ArchivedString {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq for ArchivedString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialOrd for ArchivedString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialEq<&str> for ArchivedString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<str> for ArchivedString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<ArchivedString> for &str {
    #[inline]
    fn eq(&self, other: &ArchivedString) -> bool {
        PartialEq::eq(other.as_str(), *self)
    }
}

impl PartialEq<ArchivedString> for str {
    #[inline]
    fn eq(&self, other: &ArchivedString) -> bool {
        PartialEq::eq(other.as_str(), self)
    }
}

impl PartialOrd<&str> for ArchivedString {
    #[inline]
    fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl PartialOrd<str> for ArchivedString {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<ArchivedString> for &str {
    #[inline]
    fn partial_cmp(&self, other: &ArchivedString) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.as_str())
    }
}

impl PartialOrd<ArchivedString> for str {
    #[inline]
    fn partial_cmp(&self, other: &ArchivedString) -> Option<cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

/// The resolver for `String`.
pub struct StringResolver {
    pos: usize,
}

#[cfg(feature = "bytecheck")]
mod verify {
    use bytecheck::{CheckBytes, rancor::Error};

    use crate::{string::{ArchivedString, repr::ArchivedStringRepr}, validation::{ArchiveContext, ArchiveContextExt}};

    #[inline]
    pub fn verify<C: ArchiveContext<E> + ?Sized, E: Error>(
        value: &ArchivedString,
        context: &mut C,
    ) -> Result<(), E> {
        if value.repr.is_inline() {
            unsafe {
                str::check_bytes(value.repr.as_str_ptr(), context)?;
            }
        } else {
            let base = (&value.repr as *const ArchivedStringRepr).cast();
            let offset = unsafe { value.repr.out_of_line_offset() };
            let metadata = value.repr.len();

            let ptr = unsafe {
                context.bounds_check_subtree_base_offset::<str>(base, offset, metadata)?
            };

            let range = unsafe { context.push_prefix_subtree(ptr)? };
            unsafe {
                str::check_bytes(ptr, context)?;
            }
            unsafe {
                context.pop_subtree_range(range)?;
            }
        }

        Ok(())
    }
}
