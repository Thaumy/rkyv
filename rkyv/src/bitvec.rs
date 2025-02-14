//! Archived bitwise containers.

use crate::{primitive::ArchivedUsize, vec::ArchivedVec};
use bitvec::{
    order::{BitOrder, Lsb0},
    slice::BitSlice,
    store::BitStore,
    view::BitView,
};
use core::{marker::PhantomData, ops::Deref};

/// An archived `BitVec`.
// We also have to store the bit length in the archived `BitVec`.
// This is because when calling `as_raw_slice` we will get unwanted bits if the `BitVec` bit length is not a multiple of the bit size of T.
// TODO: verify that bit_len matches the archived vector len in a verify meta
#[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
#[cfg_attr(feature = "stable_layout", repr(C))]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArchivedBitVec<T = ArchivedUsize, O = Lsb0> {
    pub(crate) inner: ArchivedVec<T>,
    pub(crate) bit_len: ArchivedUsize,
    pub(crate) _or: PhantomData<O>,
}

impl<T: BitStore, O: BitOrder> Deref for ArchivedBitVec<T, O> {
    type Target = BitSlice<T, O>;

    fn deref(&self) -> &Self::Target {
        &self.inner.view_bits::<O>()[..self.bit_len.to_native() as usize]
    }
}
