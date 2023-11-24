use std::{fmt, io, mem};

use {
    itoa, ryu,
    serde::{
        ser::{
            Error as SerdeError, Serialize, SerializeMap, SerializeSeq,
            SerializeStruct, SerializeStructVariant, SerializeTuple,
            SerializeTupleStruct, SerializeTupleVariant, Serializer,
        },
        serde_if_integer128,
    },
};

use crate::{
    error::{Error, ErrorKind},
    writer::Writer,
};

/// Serialize the given value to the given writer, and return an error if
/// anything went wrong.
pub fn serialize<S: Serialize, W: io::Write>(
    wtr: &mut Writer<W>,
    value: S,
) -> Result<(), Error> {
    value.serialize(&mut SeRecord { wtr })
}

struct SeRecord<'w, W: 'w + io::Write> {
    wtr: &'w mut Writer<W>,
}

impl<'a, 'w, W: io::Write> Serializer for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.wtr.write_field("true")
        } else {
            self.wtr.write_field("false")
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.collect_str(&v)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.collect_str(&v)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        self.wtr.write_field(buffer.format(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(value)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(&[])
    }

    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        None::<()>.serialize(self)
    }

    fn serialize_unit_struct(
        self,
        name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.wtr.write_field(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom("serializing enum tuple variants is not supported"))
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        // The right behavior for serializing maps isn't clear.
        Err(Error::custom(
            "serializing maps is not supported, \
             if you have a use case, please file an issue at \
             https://github.com/BurntSushi/rust-csv",
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom("serializing enum struct variants is not supported"))
    }
}

impl<'a, 'w, W: io::Write> SerializeSeq for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTuple for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTupleStruct for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTupleVariant for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a, 'w, W: io::Write> SerializeMap for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(
        &mut self,
        _key: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a, 'w, W: io::Write> SerializeStruct for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeStructVariant for &'a mut SeRecord<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl SerdeError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::new(ErrorKind::Serialize(msg.to_string()))
    }
}

fn error_scalar_outside_struct<T: fmt::Display>(name: T) -> Error {
    Error::custom(format!(
        "cannot serialize {} scalar outside struct \
         when writing headers from structs",
        name
    ))
}

fn error_container_inside_struct<T: fmt::Display>(name: T) -> Error {
    Error::custom(format!(
        "cannot serialize {} container inside struct \
         when writing headers from structs",
        name
    ))
}

/// Write header names corresponding to the field names of the value (if the
/// value has field names).
///
/// If the type to be serialized has field names (e.g. it's a struct), then
/// header names are written, and the `Ok` return value is `true`.
///
/// If the type to be serialized doesn't have field names, then nothing is
/// written, and the `Ok` return value is `false`.
pub fn serialize_header<S: Serialize, W: io::Write>(
    wtr: &mut Writer<W>,
    value: S,
) -> Result<bool, Error> {
    let mut ser = SeHeader::new(wtr);
    value.serialize(&mut ser).map(|_| ser.wrote_header())
}

/// State machine for `SeHeader`.
///
/// This is a diagram of the transitions in the state machine. Note that only
/// some serialization events cause a state transition, and only for certain
/// states. For example, encountering a scalar causes a transition if the state
/// is `Write` or `EncounteredStructField`, but not if the state is
/// `ErrorIfWrite(err)` or `InStructField`.
///
/// ```text
///                              +-----+
///                              |Write|
///                              +-----+
///                                 |
///              /------------------+------------------\
///              |                  |                  |
///          encounter            finish           encounter
///            scalar               |             struct field
///              |                  |                  |
///              v                  v                  v
///     +-----------------+       Ok(())        +-------------+
///     |ErrorIfWrite(err)|                     |InStructField|<--------\
///     +-----------------+                     +-------------+         |
///              |                                     |                |
///       /------+------\            /-----------------+                |
///       |             |            |                 |                |
///   encounter       finish     encounter          finish          encounter
///  struct field       |        container           field         struct field
///       |             |            |                 |                |
///       v             v            v                 v                |
///   Err(err)       Ok(())        Err(_)   +----------------------+    |
///                                         |EncounteredStructField|    |
///                                         +----------------------+    |
///                                                    |                |
///                                         /----------+----------------/
///                                         |          |
///                                     encounter    finish
///                                       scalar       |
///                                         |          |
///                                         v          v
///                                       Err(_)    Ok(())
/// ```
enum HeaderState {
    /// Start here. Headers need to be written if the type has field names.
    Write,
    /// The serializer still has not encountered a struct field. If one is
    /// encountered (headers need to be written), return the enclosed error.
    ErrorIfWrite(Error),
    /// The serializer encountered one or more struct fields (and wrote their
    /// names).
    EncounteredStructField,
    /// The serializer is currently in a struct field value.
    InStructField,
}

struct SeHeader<'w, W: 'w + io::Write> {
    wtr: &'w mut Writer<W>,
    state: HeaderState,
}

impl<'w, W: io::Write> SeHeader<'w, W> {
    fn new(wtr: &'w mut Writer<W>) -> Self {
        SeHeader { wtr, state: HeaderState::Write }
    }

    fn wrote_header(&self) -> bool {
        use self::HeaderState::*;
        match self.state {
            Write | ErrorIfWrite(_) => false,
            EncounteredStructField | InStructField => true,
        }
    }

    fn handle_scalar<T: fmt::Display>(
        &mut self,
        name: T,
    ) -> Result<(), Error> {
        use self::HeaderState::*;

        match self.state {
            Write => {
                self.state = ErrorIfWrite(error_scalar_outside_struct(name));
                Ok(())
            }
            ErrorIfWrite(_) | InStructField => Ok(()),
            EncounteredStructField => Err(error_scalar_outside_struct(name)),
        }
    }

    fn handle_container<T: fmt::Display>(
        &mut self,
        name: T,
    ) -> Result<&mut Self, Error> {
        if let HeaderState::InStructField = self.state {
            Err(error_container_inside_struct(name))
        } else {
            Ok(self)
        }
    }
}

impl<'a, 'w, W: io::Write> Serializer for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.handle_scalar(v)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.handle_scalar(v)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(v)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(value)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar("&[u8]")
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar("None")
    }

    fn serialize_some<T: ?Sized + Serialize>(
        self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar("Some(_)")
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar("()")
    }

    fn serialize_unit_struct(
        self,
        name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(format!("{}::{}", name, variant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(format!("{}(_)", name))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.handle_scalar(format!("{}::{}(_)", name, variant))
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        self.handle_container("sequence")
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        self.handle_container("tuple")
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.handle_container(name)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom("serializing enum tuple variants is not supported"))
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        // The right behavior for serializing maps isn't clear.
        Err(Error::custom(
            "serializing maps is not supported, \
             if you have a use case, please file an issue at \
             https://github.com/BurntSushi/rust-csv",
        ))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.handle_container(name)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom("serializing enum struct variants is not supported"))
    }
}

impl<'a, 'w, W: io::Write> SerializeSeq for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTuple for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTupleStruct for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeTupleVariant for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a, 'w, W: io::Write> SerializeMap for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(
        &mut self,
        _key: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a, 'w, W: io::Write> SerializeStruct for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        // Grab old state and update state to `EncounteredStructField`.
        let old_state =
            mem::replace(&mut self.state, HeaderState::EncounteredStructField);
        if let HeaderState::ErrorIfWrite(err) = old_state {
            return Err(err);
        }
        self.wtr.write_field(key)?;

        // Check that there aren't any containers in the value.
        self.state = HeaderState::InStructField;
        value.serialize(&mut **self)?;
        self.state = HeaderState::EncounteredStructField;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'w, W: io::Write> SerializeStructVariant for &'a mut SeHeader<'w, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
