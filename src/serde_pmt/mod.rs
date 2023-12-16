pub mod error;

mod serialiser;
use futuresdr::runtime::Pmt;
use serde::{Serialize, de::DeserializeOwned};
pub use serialiser::{Serializer};

use self::deserialiser::PmtDist;
mod deserialiser;


pub fn to_pmt<T>(value: &T) -> error::Result<Pmt>
where
    T: Serialize + ?Sized,
{
    let mut serializer = Serializer {};
    value.serialize(&mut serializer)
}

pub fn from_pmt<T>(value: Pmt) -> error::Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(PmtDist::from(value))
}
