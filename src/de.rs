use std::str::FromStr;

use serde::de::value::MapDeserializer;
use serde::de::{IntoDeserializer, Visitor};

use regex::Regex;

use crate::error::*;

pub(crate) struct Deserializer<'de> {
    input: &'de str,
    regex: Regex,
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de str, regex: Regex) -> Deserializer {
        Deserializer { input, regex }
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let caps = self.regex.captures(self.input).ok_or_else(Error::NoMatch)?;

        let items = self.regex.capture_names().filter_map(|n| {
            n.and_then(|name| {
                caps.name(name).map(|value| {
                    (
                        name.to_owned(),
                        Value {
                            name: name.to_owned(),
                            value: value.as_str().to_owned(),
                        },
                    )
                })
            })
        });

        let ms = MapDeserializer::new(items);

        visitor.visit_map(ms)
    }

    serde::forward_to_deserialize_any! {
        bool
        u8 u16 u32 u64
        i8 i16 i32 i64
        f32 f64
        char str string identifier
        unit seq bytes byte_buf unit_struct tuple_struct
        tuple ignored_any option newtype_struct enum struct
    }
}

struct Value {
    name: String,
    value: String,
}

impl Value {
    fn parse<T>(&self) -> Result<T>
    where
        T: FromStr,
    {
        self.value.parse().map_err(|_| self.get_parse_error())
    }

    fn get_parse_error(&self) -> Error {
        Error::BadValue {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.value.eq_ignore_ascii_case("true") {
            visitor.visit_bool(true)
        } else if self.value.eq_ignore_ascii_case("false") {
            visitor.visit_bool(false)
        } else {
            Err(self.get_parse_error())
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.value.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.value.into_deserializer())
    }

    //Remaining values can either be parsed as string or are not directly supported
    serde::forward_to_deserialize_any! {
        char str string identifier
        unit seq bytes byte_buf map unit_struct
        tuple_struct tuple ignored_any struct
    }
}
