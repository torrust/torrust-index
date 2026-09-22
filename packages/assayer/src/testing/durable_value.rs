// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Owned checkpoint values for crate-level comparison. Maps retain keys of any serde shape; sequences retain their order. Floating infinities and NaNs retain their bits instead of becoming JSON nulls.

use serde::Serialize;
use serde::ser::{self, Error as _};
use serde_json::{Error, Map, Value};

pub(super) struct DurableValue;

macro_rules! scalar {
    ($method:ident, $kind:ty) => {
        fn $method(self, value: $kind) -> Result<Value, Error> {
            serde_json::value::Serializer.$method(value)
        }
    };
}

impl ser::Serializer for DurableValue {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = Compound;
    type SerializeTuple = Compound;
    type SerializeTupleStruct = Compound;
    type SerializeTupleVariant = Compound;
    type SerializeMap = Compound;
    type SerializeStruct = Compound;
    type SerializeStructVariant = Compound;

    scalar!(serialize_bool, bool);
    scalar!(serialize_i8, i8);
    scalar!(serialize_i16, i16);
    scalar!(serialize_i32, i32);
    scalar!(serialize_i64, i64);
    scalar!(serialize_u8, u8);
    scalar!(serialize_u16, u16);
    scalar!(serialize_u32, u32);
    scalar!(serialize_u64, u64);
    scalar!(serialize_char, char);
    scalar!(serialize_str, &str);

    fn serialize_i128(self, value: i128) -> Result<Value, Error> {
        Ok(Value::String(format!("i128:{value}")))
    }
    fn serialize_u128(self, value: u128) -> Result<Value, Error> {
        Ok(Value::String(format!("u128:{value}")))
    }
    fn serialize_f32(self, value: f32) -> Result<Value, Error> {
        self.serialize_f64(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> Result<Value, Error> {
        if value.is_finite() {
            serde_json::value::Serializer.serialize_f64(value)
        } else {
            Ok(Value::String(format!("nonfinite:{:016x}", value.to_bits())))
        }
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<Value, Error> {
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, Error> {
        Ok(Value::Null)
    }
    fn serialize_unit_variant(self, _name: &'static str, _index: u32, variant: &'static str) -> Result<Value, Error> {
        Ok(Value::String(variant.to_owned()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result<Value, Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, Error> {
        Ok(Value::Object(Map::from_iter([(variant.to_owned(), value.serialize(self)?)])))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound, Error> {
        Ok(Compound::default())
    }
    fn serialize_tuple(self, _len: usize) -> Result<Compound, Error> {
        Ok(Compound::default())
    }
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Compound, Error> {
        Ok(Compound::default())
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound, Error> {
        Ok(Compound {
            variant: Some(variant),
            ..Compound::default()
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Compound, Error> {
        Ok(Compound {
            map: true,
            ..Compound::default()
        })
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Compound, Error> {
        Ok(Compound {
            record: true,
            ..Compound::default()
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound, Error> {
        Ok(Compound {
            record: true,
            variant: Some(variant),
            ..Compound::default()
        })
    }
}

#[derive(Default)]
pub(super) struct Compound {
    items: Vec<Value>,
    fields: Map<String, Value>,
    key: Option<Value>,
    map: bool,
    record: bool,
    variant: Option<&'static str>,
}

impl Compound {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(value.serialize(DurableValue)?);
        Ok(())
    }

    fn field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Error> {
        self.fields.insert(key.to_owned(), value.serialize(DurableValue)?);
        Ok(())
    }

    fn finish(mut self) -> Result<Value, Error> {
        if self.key.is_some() {
            return Err(Error::custom("durable map ended without its value"));
        }
        if self.map {
            // Association order is not durable state. Preserve complete keys and values, sorting only map entries; ordinary sequences are never sorted here.
            self.items.sort_by_cached_key(|pair| pair[0].to_string());
        }
        let value = if self.record {
            Value::Object(self.fields)
        } else {
            Value::Array(self.items)
        };
        Ok(match self.variant {
            Some(variant) => Value::Object(Map::from_iter([(variant.to_owned(), value)])),
            None => value,
        })
    }
}

macro_rules! sequence {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for Compound {
            type Ok = Value;
            type Error = Error;
            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
                self.element(value)
            }
            fn end(self) -> Result<Value, Error> {
                self.finish()
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);

macro_rules! record {
    ($trait:ident) => {
        impl ser::$trait for Compound {
            type Ok = Value;
            type Error = Error;
            fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Error> {
                self.field(key, value)
            }
            fn end(self) -> Result<Value, Error> {
                self.finish()
            }
        }
    };
}
record!(SerializeStruct);
record!(SerializeStructVariant);

impl ser::SerializeMap for Compound {
    type Ok = Value;
    type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        if self.key.is_some() {
            return Err(Error::custom("durable map received two keys"));
        }
        self.key = Some(key.serialize(DurableValue)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let key = self.key.take().ok_or_else(|| Error::custom("durable map value has no key"))?;
        self.items.push(Value::Array(vec![key, value.serialize(DurableValue)?]));
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        self.finish()
    }
}
