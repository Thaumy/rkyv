use crate::{
    ser::{ScratchSpace, Serializer},
    vec::{ArchivedVec, VecResolver},
    Archive, Deserialize, DeserializeUnsized, Serialize,
};
#[cfg(not(feature = "std"))]
use alloc::{alloc, boxed::Box, vec::Vec};
use core::cmp;
#[cfg(feature = "std")]
use std::alloc;

impl<T: PartialEq<U>, U> PartialEq<Vec<U>> for ArchivedVec<T> {
    #[inline]
    fn eq(&self, other: &Vec<U>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U> PartialEq<ArchivedVec<U>> for Vec<T> {
    #[inline]
    fn eq(&self, other: &ArchivedVec<U>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: PartialOrd> PartialOrd<Vec<T>> for ArchivedVec<T> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<T>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: PartialOrd> PartialOrd<ArchivedVec<T>> for Vec<T> {
    #[inline]
    fn partial_cmp(&self, other: &ArchivedVec<T>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Archive> Archive for Vec<T> {
    type Archived = ArchivedVec<T::Archived>;
    type Resolver = VecResolver;

    #[inline]
    unsafe fn resolve(
        &self,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        ArchivedVec::resolve_from_slice(self.as_slice(), pos, resolver, out);
    }
}

impl<T: Serialize<S, E>, S: ScratchSpace<E> + Serializer<E> + ?Sized, E> Serialize<S, E>
    for Vec<T>
{
    #[inline]
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, E> {
        ArchivedVec::<T::Archived>::serialize_from_slice(
            self.as_slice(),
            serializer,
        )
    }
}

impl<T: Archive, D: ?Sized, E> Deserialize<Vec<T>, D, E>
    for ArchivedVec<T::Archived>
where
    [T::Archived]: DeserializeUnsized<[T], D, E>,
{
    #[inline]
    fn deserialize(&self, deserializer: &mut D) -> Result<Vec<T>, E> {
        unsafe {
            let data_address = self
                .as_slice()
                .deserialize_unsized(deserializer, |layout| {
                    alloc::alloc(layout)
                })?;
            let metadata =
                self.as_slice().deserialize_metadata(deserializer)?;
            let ptr = ptr_meta::from_raw_parts_mut(data_address, metadata);
            Ok(Box::<[T]>::from_raw(ptr).into())
        }
    }
}
