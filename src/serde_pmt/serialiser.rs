use core::fmt::{self, Display};
use futuresdr::runtime::Pmt;
use serde::{
    ser::{self, Impossible},
    Serialize,
};
use std::collections::HashMap;

use super::{error::{Error, Result}, to_pmt};

pub struct Serializer {}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = Pmt;

    type Error = Error;

    type SerializeSeq = Impossible<Pmt, Error>; // TODO

    type SerializeTuple = Impossible<Pmt, Error>; // TODO

    type SerializeTupleStruct = Impossible<Pmt, Error>; // TODO

    type SerializeTupleVariant = Impossible<Pmt, Error>; // TODO

    type SerializeMap = SerializeMap;

    type SerializeStruct = SerializeMap;

    type SerializeStructVariant = Impossible<Pmt, Error>; // TODO

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Pmt::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_f32(v as f32)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_f32(v as f32)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_f32(v as f32)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_f32(v as f32)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u32(v as u32)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u32(v as u32)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(Pmt::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(Pmt::U64(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Pmt::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Pmt::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut s = String::new();
        s.push(v);
        Ok(Pmt::String(s))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Pmt::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let vec = v.iter().map(|&b| Pmt::U32(b.into())).collect();
        Ok(Pmt::VecPmt(vec))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Pmt::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Pmt::Null)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> std::prelude::v1::Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let mut values = HashMap::<String, Pmt>::new();
        values.insert(String::from(variant), to_pmt(value)?);
        Ok(Pmt::MapStrPmt(values))
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::prelude::v1::Result<Self::SerializeSeq, Self::Error> {
        // Ok(SerializeVec {
        //     vec: Vec::with_capacity(len.unwrap_or(0)),
        // })
        todo!()
    }

    fn serialize_tuple(
        self,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeTupleVariant, Self::Error> {
        // Ok(SerializeTupleVariant {
        //     name: String::from(variant),
        //     vec: Vec::with_capacity(len),
        // })
        todo!()
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> std::prelude::v1::Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::prelude::v1::Result<Self::SerializeStructVariant, Self::Error> {
        // Ok(SerializeStructVariant {
        //     name: String::from(variant),
        //     map: Map::new(),
        // })
        todo!()
    }

    fn serialize_i128(self, v: i128) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(ser::Error::custom("i128 is not supported"))
    }

    fn serialize_u128(self, v: u128) -> std::prelude::v1::Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(ser::Error::custom("u128 is not supported"))
    }

    fn collect_str<T>(self, value: &T) -> Result<Pmt>
    where
        T: ?Sized + Display,
    {
        Ok(Pmt::String(value.to_string()))
    }
}

pub struct SerializeMap {
    map: HashMap<String, Pmt>,
    next_key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Pmt;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, to_pmt(value)?);
        Ok(())
    }

    fn end(self) -> Result<Pmt> {
        Ok(Pmt::MapStrPmt(self.map))
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::KeyMustBeAString
}

fn float_key_must_be_finite() -> Error {
    Error::FloatKeyMustBeFinite
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, value: bool) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String> {
        if value.is_finite() {
            Ok(format!("{:?}", value))
        } else {
            Err(float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String> {
        if value.is_finite() {
            Ok(format!("{:?}", value))
        } else {
            Err(float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Pmt;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Pmt> {
        serde::ser::SerializeMap::end(self)
    }
}
