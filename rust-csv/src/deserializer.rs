use std::{error::Error as StdError, fmt, iter, num, str};

use serde::{
    de::value::BorrowedBytesDeserializer,
    de::{
        Deserialize, DeserializeSeed, Deserializer, EnumAccess,
        Error as SerdeError, IntoDeserializer, MapAccess, SeqAccess,
        Unexpected, VariantAccess, Visitor,
    },
    serde_if_integer128,
};

use crate::{
    byte_record::{ByteRecord, ByteRecordIter},
    error::{Error, ErrorKind},
    string_record::{StringRecord, StringRecordIter},
};

use self::DeserializeErrorKind as DEK;

pub fn deserialize_string_record<'de, D: Deserialize<'de>>(
    record: &'de StringRecord,
    headers: Option<&'de StringRecord>,
) -> Result<D, Error> {
    let mut deser = DeRecordWrap(DeStringRecord {
        it: record.iter().peekable(),
        headers: headers.map(|r| r.iter()),
        field: 0,
    });
    D::deserialize(&mut deser).map_err(|err| {
        Error::new(ErrorKind::Deserialize {
            pos: record.position().map(Clone::clone),
            err,
        })
    })
}

pub fn deserialize_byte_record<'de, D: Deserialize<'de>>(
    record: &'de ByteRecord,
    headers: Option<&'de ByteRecord>,
) -> Result<D, Error> {
    let mut deser = DeRecordWrap(DeByteRecord {
        it: record.iter().peekable(),
        headers: headers.map(|r| r.iter()),
        field: 0,
    });
    D::deserialize(&mut deser).map_err(|err| {
        Error::new(ErrorKind::Deserialize {
            pos: record.position().map(Clone::clone),
            err,
        })
    })
}

/// An over-engineered internal trait that permits writing a single Serde
/// deserializer that works on both ByteRecord and StringRecord.
///
/// We *could* implement a single deserializer on `ByteRecord` and simply
/// convert `StringRecord`s to `ByteRecord`s, but then the implementation
/// would be required to redo UTF-8 validation checks in certain places.
///
/// How does this work? We create a new `DeRecordWrap` type that wraps
/// either a `StringRecord` or a `ByteRecord`. We then implement
/// `DeRecord` for `DeRecordWrap<ByteRecord>` and `DeRecordWrap<StringRecord>`.
/// Finally, we impl `serde::Deserialize` for `DeRecordWrap<T>` where
/// `T: DeRecord`. That is, the `DeRecord` type corresponds to the differences
/// between deserializing into a `ByteRecord` and deserializing into a
/// `StringRecord`.
///
/// The lifetime `'r` refers to the lifetime of the underlying record.
trait DeRecord<'r> {
    /// Returns true if and only if this deserialize has access to headers.
    fn has_headers(&self) -> bool;

    /// Extracts the next string header value from the underlying record.
    fn next_header(&mut self) -> Result<Option<&'r str>, DeserializeError>;

    /// Extracts the next raw byte header value from the underlying record.
    fn next_header_bytes(
        &mut self,
    ) -> Result<Option<&'r [u8]>, DeserializeError>;

    /// Extracts the next string field from the underlying record.
    fn next_field(&mut self) -> Result<&'r str, DeserializeError>;

    /// Extracts the next raw byte field from the underlying record.
    fn next_field_bytes(&mut self) -> Result<&'r [u8], DeserializeError>;

    /// Peeks at the next field from the underlying record.
    fn peek_field(&mut self) -> Option<&'r [u8]>;

    /// Returns an error corresponding to the most recently extracted field.
    fn error(&self, kind: DeserializeErrorKind) -> DeserializeError;

    /// Infer the type of the next field and deserialize it.
    fn infer_deserialize<'de, V: Visitor<'de>>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, DeserializeError>;
}

struct DeRecordWrap<T>(T);

impl<'r, T: DeRecord<'r>> DeRecord<'r> for DeRecordWrap<T> {
    #[inline]
    fn has_headers(&self) -> bool {
        self.0.has_headers()
    }

    #[inline]
    fn next_header(&mut self) -> Result<Option<&'r str>, DeserializeError> {
        self.0.next_header()
    }

    #[inline]
    fn next_header_bytes(
        &mut self,
    ) -> Result<Option<&'r [u8]>, DeserializeError> {
        self.0.next_header_bytes()
    }

    #[inline]
    fn next_field(&mut self) -> Result<&'r str, DeserializeError> {
        self.0.next_field()
    }

    #[inline]
    fn next_field_bytes(&mut self) -> Result<&'r [u8], DeserializeError> {
        self.0.next_field_bytes()
    }

    #[inline]
    fn peek_field(&mut self) -> Option<&'r [u8]> {
        self.0.peek_field()
    }

    #[inline]
    fn error(&self, kind: DeserializeErrorKind) -> DeserializeError {
        self.0.error(kind)
    }

    #[inline]
    fn infer_deserialize<'de, V: Visitor<'de>>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        self.0.infer_deserialize(visitor)
    }
}

struct DeStringRecord<'r> {
    it: iter::Peekable<StringRecordIter<'r>>,
    headers: Option<StringRecordIter<'r>>,
    field: u64,
}

impl<'r> DeRecord<'r> for DeStringRecord<'r> {
    #[inline]
    fn has_headers(&self) -> bool {
        self.headers.is_some()
    }

    #[inline]
    fn next_header(&mut self) -> Result<Option<&'r str>, DeserializeError> {
        Ok(self.headers.as_mut().and_then(|it| it.next()))
    }

    #[inline]
    fn next_header_bytes(
        &mut self,
    ) -> Result<Option<&'r [u8]>, DeserializeError> {
        Ok(self.next_header()?.map(|s| s.as_bytes()))
    }

    #[inline]
    fn next_field(&mut self) -> Result<&'r str, DeserializeError> {
        match self.it.next() {
            Some(field) => {
                self.field += 1;
                Ok(field)
            }
            None => Err(DeserializeError {
                field: None,
                kind: DEK::UnexpectedEndOfRow,
            }),
        }
    }

    #[inline]
    fn next_field_bytes(&mut self) -> Result<&'r [u8], DeserializeError> {
        self.next_field().map(|s| s.as_bytes())
    }

    #[inline]
    fn peek_field(&mut self) -> Option<&'r [u8]> {
        self.it.peek().map(|s| s.as_bytes())
    }

    fn error(&self, kind: DeserializeErrorKind) -> DeserializeError {
        DeserializeError { field: Some(self.field.saturating_sub(1)), kind }
    }

    fn infer_deserialize<'de, V: Visitor<'de>>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        let x = self.next_field()?;
        if x == "true" {
            return visitor.visit_bool(true);
        } else if x == "false" {
            return visitor.visit_bool(false);
        } else if let Some(n) = try_positive_integer64(x) {
            return visitor.visit_u64(n);
        } else if let Some(n) = try_negative_integer64(x) {
            return visitor.visit_i64(n);
        }
        serde_if_integer128! {
            if let Some(n) = try_positive_integer128(x) {
                return visitor.visit_u128(n);
            } else if let Some(n) = try_negative_integer128(x) {
                return visitor.visit_i128(n);
            }
        }
        if let Some(n) = try_float(x) {
            visitor.visit_f64(n)
        } else {
            visitor.visit_str(x)
        }
    }
}

struct DeByteRecord<'r> {
    it: iter::Peekable<ByteRecordIter<'r>>,
    headers: Option<ByteRecordIter<'r>>,
    field: u64,
}

impl<'r> DeRecord<'r> for DeByteRecord<'r> {
    #[inline]
    fn has_headers(&self) -> bool {
        self.headers.is_some()
    }

    #[inline]
    fn next_header(&mut self) -> Result<Option<&'r str>, DeserializeError> {
        match self.next_header_bytes() {
            Ok(Some(field)) => Ok(Some(
                str::from_utf8(field)
                    .map_err(|err| self.error(DEK::InvalidUtf8(err)))?,
            )),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    #[inline]
    fn next_header_bytes(
        &mut self,
    ) -> Result<Option<&'r [u8]>, DeserializeError> {
        Ok(self.headers.as_mut().and_then(|it| it.next()))
    }

    #[inline]
    fn next_field(&mut self) -> Result<&'r str, DeserializeError> {
        self.next_field_bytes().and_then(|field| {
            str::from_utf8(field)
                .map_err(|err| self.error(DEK::InvalidUtf8(err)))
        })
    }

    #[inline]
    fn next_field_bytes(&mut self) -> Result<&'r [u8], DeserializeError> {
        match self.it.next() {
            Some(field) => {
                self.field += 1;
                Ok(field)
            }
            None => Err(DeserializeError {
                field: None,
                kind: DEK::UnexpectedEndOfRow,
            }),
        }
    }

    #[inline]
    fn peek_field(&mut self) -> Option<&'r [u8]> {
        self.it.peek().map(|s| *s)
    }

    fn error(&self, kind: DeserializeErrorKind) -> DeserializeError {
        DeserializeError { field: Some(self.field.saturating_sub(1)), kind }
    }

    fn infer_deserialize<'de, V: Visitor<'de>>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, DeserializeError> {
        let x = self.next_field_bytes()?;
        if x == b"true" {
            return visitor.visit_bool(true);
        } else if x == b"false" {
            return visitor.visit_bool(false);
        } else if let Some(n) = try_positive_integer64_bytes(x) {
            return visitor.visit_u64(n);
        } else if let Some(n) = try_negative_integer64_bytes(x) {
            return visitor.visit_i64(n);
        }
        serde_if_integer128! {
            if let Some(n) = try_positive_integer128_bytes(x) {
                return visitor.visit_u128(n);
            } else if let Some(n) = try_negative_integer128_bytes(x) {
                return visitor.visit_i128(n);
            }
        }
        if let Some(n) = try_float_bytes(x) {
            visitor.visit_f64(n)
        } else if let Ok(s) = str::from_utf8(x) {
            visitor.visit_str(s)
        } else {
            visitor.visit_bytes(x)
        }
    }
}

macro_rules! deserialize_int {
    ($method:ident, $visit:ident, $inttype:ty) => {
        fn $method<V: Visitor<'de>>(
            self,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            let field = self.next_field()?;
            let num = if field.starts_with("0x") {
                <$inttype>::from_str_radix(&field[2..], 16)
            } else {
                field.parse()
            };
            visitor.$visit(num.map_err(|err| self.error(DEK::ParseInt(err)))?)
        }
    };
}

impl<'a, 'de: 'a, T: DeRecord<'de>> Deserializer<'de>
    for &'a mut DeRecordWrap<T>
{
    type Error = DeserializeError;

    fn deserialize_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.infer_deserialize(visitor)
    }

    fn deserialize_bool<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(
            self.next_field()?
                .parse()
                .map_err(|err| self.error(DEK::ParseBool(err)))?,
        )
    }

    deserialize_int!(deserialize_u8, visit_u8, u8);
    deserialize_int!(deserialize_u16, visit_u16, u16);
    deserialize_int!(deserialize_u32, visit_u32, u32);
    deserialize_int!(deserialize_u64, visit_u64, u64);
    serde_if_integer128! {
        deserialize_int!(deserialize_u128, visit_u128, u128);
    }
    deserialize_int!(deserialize_i8, visit_i8, i8);
    deserialize_int!(deserialize_i16, visit_i16, i16);
    deserialize_int!(deserialize_i32, visit_i32, i32);
    deserialize_int!(deserialize_i64, visit_i64, i64);
    serde_if_integer128! {
        deserialize_int!(deserialize_i128, visit_i128, i128);
    }

    fn deserialize_f32<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(
            self.next_field()?
                .parse()
                .map_err(|err| self.error(DEK::ParseFloat(err)))?,
        )
    }

    fn deserialize_f64<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(
            self.next_field()?
                .parse()
                .map_err(|err| self.error(DEK::ParseFloat(err)))?,
        )
    }

    fn deserialize_char<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let field = self.next_field()?;
        let len = field.chars().count();
        if len != 1 {
            return Err(self.error(DEK::Message(format!(
                "expected single character but got {} characters in '{}'",
                len, field
            ))));
        }
        visitor.visit_char(field.chars().next().unwrap())
    }

    fn deserialize_str<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.next_field().and_then(|f| visitor.visit_borrowed_str(f))
    }

    fn deserialize_string<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.next_field().and_then(|f| visitor.visit_str(f.into()))
    }

    fn deserialize_bytes<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.next_field_bytes().and_then(|f| visitor.visit_borrowed_bytes(f))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.next_field_bytes()
            .and_then(|f| visitor.visit_byte_buf(f.to_vec()))
    }

    fn deserialize_option<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.peek_field() {
            None => visitor.visit_none(),
            Some(f) if f.is_empty() => {
                self.next_field().expect("empty field");
                visitor.visit_none()
            }
            Some(_) => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }

    fn deserialize_map<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.has_headers() {
            visitor.visit_seq(self)
        } else {
            visitor.visit_map(self)
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.has_headers() {
            visitor.visit_seq(self)
        } else {
            visitor.visit_map(self)
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(self.error(DEK::Unsupported("deserialize_identifier".into())))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Read and drop the next field.
        // This code is reached, e.g., when trying to deserialize a header
        // that doesn't exist in the destination struct.
        let _ = self.next_field_bytes()?;
        visitor.visit_unit()
    }
}

impl<'a, 'de: 'a, T: DeRecord<'de>> EnumAccess<'de>
    for &'a mut DeRecordWrap<T>
{
    type Error = DeserializeError;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant_name = self.next_field()?;
        seed.deserialize(variant_name.into_deserializer()).map(|v| (v, self))
    }
}

impl<'a, 'de: 'a, T: DeRecord<'de>> VariantAccess<'de>
    for &'a mut DeRecordWrap<T>
{
    type Error = DeserializeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<U: DeserializeSeed<'de>>(
        self,
        _seed: U,
    ) -> Result<U::Value, Self::Error> {
        let unexp = Unexpected::UnitVariant;
        Err(DeserializeError::invalid_type(unexp, &"newtype variant"))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let unexp = Unexpected::UnitVariant;
        Err(DeserializeError::invalid_type(unexp, &"tuple variant"))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let unexp = Unexpected::UnitVariant;
        Err(DeserializeError::invalid_type(unexp, &"struct variant"))
    }
}

impl<'a, 'de: 'a, T: DeRecord<'de>> SeqAccess<'de>
    for &'a mut DeRecordWrap<T>
{
    type Error = DeserializeError;

    fn next_element_seed<U: DeserializeSeed<'de>>(
        &mut self,
        seed: U,
    ) -> Result<Option<U::Value>, Self::Error> {
        if self.peek_field().is_none() {
            Ok(None)
        } else {
            seed.deserialize(&mut **self).map(Some)
        }
    }
}

impl<'a, 'de: 'a, T: DeRecord<'de>> MapAccess<'de>
    for &'a mut DeRecordWrap<T>
{
    type Error = DeserializeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        assert!(self.has_headers());
        let field = match self.next_header_bytes()? {
            None => return Ok(None),
            Some(field) => field,
        };
        seed.deserialize(BorrowedBytesDeserializer::new(field)).map(Some)
    }

    fn next_value_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<K::Value, Self::Error> {
        seed.deserialize(&mut **self)
    }
}

/// An Serde deserialization error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeserializeError {
    field: Option<u64>,
    kind: DeserializeErrorKind,
}

/// The type of a Serde deserialization error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeserializeErrorKind {
    /// A generic Serde deserialization error.
    Message(String),
    /// A generic Serde unsupported error.
    Unsupported(String),
    /// This error occurs when a Rust type expects to decode another field
    /// from a row, but no more fields exist.
    UnexpectedEndOfRow,
    /// This error occurs when UTF-8 validation on a field fails. UTF-8
    /// validation is only performed when the Rust type requires it (e.g.,
    /// a `String` or `&str` type).
    InvalidUtf8(str::Utf8Error),
    /// This error occurs when a boolean value fails to parse.
    ParseBool(str::ParseBoolError),
    /// This error occurs when an integer value fails to parse.
    ParseInt(num::ParseIntError),
    /// This error occurs when a float value fails to parse.
    ParseFloat(num::ParseFloatError),
}

impl SerdeError for DeserializeError {
    fn custom<T: fmt::Display>(msg: T) -> DeserializeError {
        DeserializeError { field: None, kind: DEK::Message(msg.to_string()) }
    }
}

impl StdError for DeserializeError {
    fn description(&self) -> &str {
        self.kind.description()
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(field) = self.field {
            write!(f, "field {}: {}", field, self.kind)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl fmt::Display for DeserializeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DeserializeErrorKind::*;

        match *self {
            Message(ref msg) => write!(f, "{}", msg),
            Unsupported(ref which) => {
                write!(f, "unsupported deserializer method: {}", which)
            }
            UnexpectedEndOfRow => write!(f, "{}", self.description()),
            InvalidUtf8(ref err) => err.fmt(f),
            ParseBool(ref err) => err.fmt(f),
            ParseInt(ref err) => err.fmt(f),
            ParseFloat(ref err) => err.fmt(f),
        }
    }
}

impl DeserializeError {
    /// Return the field index (starting at 0) of this error, if available.
    pub fn field(&self) -> Option<u64> {
        self.field
    }

    /// Return the underlying error kind.
    pub fn kind(&self) -> &DeserializeErrorKind {
        &self.kind
    }
}

impl DeserializeErrorKind {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        use self::DeserializeErrorKind::*;

        match *self {
            Message(_) => "deserialization error",
            Unsupported(_) => "unsupported deserializer method",
            UnexpectedEndOfRow => "expected field, but got end of row",
            InvalidUtf8(ref err) => err.description(),
            ParseBool(ref err) => err.description(),
            ParseInt(ref err) => err.description(),
            ParseFloat(ref err) => err.description(),
        }
    }
}

serde_if_integer128! {
    fn try_positive_integer128(s: &str) -> Option<u128> {
        s.parse().ok()
    }

    fn try_negative_integer128(s: &str) -> Option<i128> {
        s.parse().ok()
    }
}

fn try_positive_integer64(s: &str) -> Option<u64> {
    s.parse().ok()
}

fn try_negative_integer64(s: &str) -> Option<i64> {
    s.parse().ok()
}

fn try_float(s: &str) -> Option<f64> {
    s.parse().ok()
}

fn try_positive_integer64_bytes(s: &[u8]) -> Option<u64> {
    str::from_utf8(s).ok().and_then(|s| s.parse().ok())
}

fn try_negative_integer64_bytes(s: &[u8]) -> Option<i64> {
    str::from_utf8(s).ok().and_then(|s| s.parse().ok())
}

serde_if_integer128! {
    fn try_positive_integer128_bytes(s: &[u8]) -> Option<u128> {
        str::from_utf8(s).ok().and_then(|s| s.parse().ok())
    }

    fn try_negative_integer128_bytes(s: &[u8]) -> Option<i128> {
        str::from_utf8(s).ok().and_then(|s| s.parse().ok())
    }
}

fn try_float_bytes(s: &[u8]) -> Option<f64> {
    str::from_utf8(s).ok().and_then(|s| s.parse().ok())
}
