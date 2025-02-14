#[cfg(feature = "alloc")]
pub mod alloc {
    use crate::util::alloc::*;
    use rkyv::{
        access,
        bytecheck::CheckBytes,
        rancor::{Error, Strategy},
        validation::validators::DefaultValidator,
        Serialize,
    };

    pub fn serialize_and_check<
        T: Serialize<Strategy<DefaultSerializer, E>>,
        E: Error,
    >(
        value: &T,
    ) where
        T::Archived: for<'a> CheckBytes<Strategy<DefaultValidator, E>>,
    {
        let buf =
            rkyv::util::serialize_into(value, DefaultSerializer::default())
                .expect("failed to archive value")
                .into_writer();

        access::<T, E>(buf.as_ref()).unwrap();
    }
}
