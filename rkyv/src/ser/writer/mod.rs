//! Writing backends for serializers.

#[cfg(feature = "alloc")]
mod alloc;
mod core;
#[cfg(feature = "std")]
mod std;

#[cfg(feature = "alloc")]
pub use self::alloc::*;
pub use self::core::*;
#[cfg(feature = "std")]
pub use self::std::*;

use ::core::{mem, slice};
use rancor::{Fallible, Strategy};

use crate::{Archive, ArchiveUnsized, RelPtr};

/// A writer that knows its current position.
pub trait Positional {
    /// Returns the current position of the writer.
    fn pos(&self) -> usize;
}

impl<T, E> Positional for Strategy<T, E>
where
    T: Positional + ?Sized,
{
    fn pos(&self) -> usize {
        T::pos(self)
    }
}

/// A type that writes bytes to some output.
///
/// A type that is [`Write`](::std::io::Write) can be wrapped in an [`IoWriter`]
/// to equip it with `Write`.
///
/// It's important that the memory for archived objects is properly aligned
/// before attempting to read objects out of it; use an
/// [`AlignedVec`](crate::util::AlignedVec) or the
/// [`AlignedBytes`](crate::util::AlignedBytes) wrappers as appropriate.
pub trait Writer<E = <Self as Fallible>::Error>: Positional {
    /// Attempts to write the given bytes to the serializer.
    fn write(&mut self, bytes: &[u8]) -> Result<(), E>;
}

impl<T, E> Writer<E> for Strategy<T, E>
where
    T: Writer<E> + ?Sized,
{
    fn write(&mut self, bytes: &[u8]) -> Result<(), E> {
        T::write(self, bytes)
    }
}

/// TODO: Document
pub trait WriterExt<E>: Writer<E> {
    /// Advances the given number of bytes as padding.
    #[inline]
    fn pad(&mut self, padding: usize) -> Result<(), E> {
        const MAX_ZEROES: usize = 32;
        const ZEROES: [u8; MAX_ZEROES] = [0; MAX_ZEROES];
        debug_assert!(padding < MAX_ZEROES);

        self.write(&ZEROES[0..padding])
    }

    /// Aligns the position of the serializer to the given alignment.
    #[inline]
    fn align(&mut self, align: usize) -> Result<usize, E> {
        let mask = align - 1;
        debug_assert_eq!(align & mask, 0);

        self.pad((align - (self.pos() & mask)) & mask)?;
        Ok(self.pos())
    }

    /// Aligns the position of the serializer to be suitable to write the given type.
    #[inline]
    fn align_for<T>(&mut self) -> Result<usize, E> {
        self.align(mem::align_of::<T>())
    }

    /// Resolves the given value with its resolver and writes the archived type.
    ///
    /// Returns the position of the written archived type.
    ///
    /// # Safety
    ///
    /// - `resolver` must be the result of serializing `value`
    /// - The serializer must be aligned for a `T::Archived`
    unsafe fn resolve_aligned<T: Archive + ?Sized>(
        &mut self,
        value: &T,
        resolver: T::Resolver,
    ) -> Result<usize, E> {
        let pos = self.pos();
        debug_assert_eq!(pos & (mem::align_of::<T::Archived>() - 1), 0);

        let mut resolved = mem::MaybeUninit::<T::Archived>::uninit();
        resolved.as_mut_ptr().write_bytes(0, 1);
        value.resolve(pos, resolver, resolved.as_mut_ptr());

        let data = resolved.as_ptr().cast::<u8>();
        let len = mem::size_of::<T::Archived>();
        self.write(slice::from_raw_parts(data, len))?;
        Ok(pos)
    }

    /// Resolves the given reference with its resolver and writes the archived reference.
    ///
    /// Returns the position of the written archived `RelPtr`.
    ///
    /// # Safety
    ///
    /// The serializer must be aligned for a `RelPtr<T::Archived>`.
    unsafe fn resolve_unsized_aligned<T: ArchiveUnsized + ?Sized>(
        &mut self,
        value: &T,
        to: usize,
    ) -> Result<usize, E> {
        let from = self.pos();
        debug_assert_eq!(
            from & (mem::align_of::<RelPtr<T::Archived>>() - 1),
            0
        );

        let mut resolved = mem::MaybeUninit::<RelPtr<T::Archived>>::uninit();
        resolved.as_mut_ptr().write_bytes(0, 1);
        RelPtr::emplace_unsized(
            from,
            to,
            value.archived_metadata(),
            resolved.as_mut_ptr(),
        );

        let data = resolved.as_ptr().cast::<u8>();
        let len = mem::size_of::<RelPtr<T::Archived>>();
        self.write(slice::from_raw_parts(data, len))?;
        Ok(from)
    }
}

impl<T, E> WriterExt<E> for T where T: Writer<E> + ?Sized {}
