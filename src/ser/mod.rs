use itoa::Integer;
use paste::paste;
use serde::ser::{
	self, Serialize, SerializeSeq, SerializeStruct, SerializeTuple,
	SerializeTupleStruct
};

mod error;
pub use error::Error;

pub struct Serializer {
	/// Buffer that the output gets written to.
	buf: String,

	/// Set to true for pretty output.
	pretty: bool,

	/// The current level of nesting
	lvl: usize,

	/// The indentation (in levels) of the current line
	indent: usize,

	/// An itoa::Buffer to re-use when printing integers
	itoa_buffer: itoa::Buffer
}

impl Serializer {
	fn new(pretty: bool) -> Self {
		Self {
			buf: String::new(),
			pretty,
			lvl: 0,
			indent: 0,
			itoa_buffer: itoa::Buffer::new()
		}
	}
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn to_string<T>(value: &T) -> Result<String>
where
	T: ?Sized + Serialize
{
	let mut serializer = Serializer::new(false);
	value.serialize(&mut serializer)?;
	Ok(serializer.buf)
}

pub fn to_string_pretty<T>(value: &T) -> Result<String>
where
	T: ?Sized + Serialize
{
	let mut serializer = Serializer::new(true);
	value.serialize(&mut serializer)?;
	Ok(serializer.buf)
}

impl Serializer {
	fn newline(&mut self) {
		self.buf += "\n";
		for _ in 0..self.lvl {
			self.buf += "  ";
		}
		self.indent = self.lvl;
	}

	fn begin_sexpr(&mut self, name: &str) {
		if self.lvl > 0 {
			if self.pretty {
				self.newline();
			} else {
				self.buf += " ";
			}
		}
		self.lvl += 1;
		self.buf += "(";
		self.buf += name;
	}

	fn end_sexpr(&mut self) {
		self.lvl -= 1;
		self.buf += ")";
	}

	fn write_integer<I: Integer>(&mut self, v: I) {
		self.buf += " ";
		self.buf += self.itoa_buffer.format(v);
	}

	fn write_float<F: ToString>(&mut self, v: F) {
		self.buf += " ";
		self.buf += &v.to_string();
	}

	fn write_str(&mut self, v: &str, aggressive_quotes: bool) {
		self.buf += " ";

		const CHARS: &[char] = &[' ', '\t', '\n', '\r', '(', ')', '"'];
		let need_quotes = v.is_empty()
			|| match aggressive_quotes {
				true => v.chars().any(|ch| !ch.is_ascii_alphabetic() && ch != '_'),
				false => v.contains(CHARS)
			};

		if need_quotes {
			self.buf += r#"""#;
			self.buf += &v.replace('\\', r"\\").replace('"', r#"\""#);
			self.buf += r#"""#;
		} else {
			self.buf += v;
		}
	}
}

macro_rules! serialize_type_error {
	($(fn $ident:ident $(<$T:ident>)? (self $(, $arg_ty:ty)*);)+) => {
		$(
			fn $ident $(<$T>)? (self $(, _: $arg_ty)*) -> Result<Self::Ok, Self::Error>
			$(where $T: ?Sized + Serialize)?
			{
				Err(Error::ExpectedStruct)
			}
		)+
	};

	($(fn $ident:ident $(<$T:ident>)? (self $(, $arg_ty:ty)*) -> $ret:ty;)+) => {
		$(
			fn $ident $(<$T>)? (self $(, _: $arg_ty)*) -> $ret
			$(where $T: ?Sized + Serialize)?
			{
				Err(Error::ExpectedStruct)
			}
		)+
	};

	($(fn $ident:ident $(<$T:ident>)? (self $(, $arg_ty:ty)*) = $err:expr;)+) => {
		$(
			fn $ident $(<$T>)? (self $(, _: $arg_ty)*) -> Result<Self::Ok, Self::Error>
			$(where $T: ?Sized + Serialize)?
			{
				Err($err)
			}
		)+
	};

	($(fn $ident:ident $(<$T:ident>)? (self $(, $arg_ty:ty)*) -> $ret:ty = $err:expr;)+) => {
		$(
			fn $ident $(<$T>)? (self $(, _: $arg_ty)*) -> $ret
			$(where $T: ?Sized + Serialize)?
			{
				Err($err)
			}
		)+
	};
}

type Impossible<T = (), E = Error> = serde::ser::Impossible<T, E>;

impl<'a> ser::Serializer for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Impossible;
	type SerializeTuple = Impossible;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Impossible;
	type SerializeMap = Impossible;
	type SerializeStruct = Self;
	type SerializeStructVariant = Impossible;

	serialize_type_error! {
		fn serialize_bool(self, bool);
		fn serialize_i8(self, i8);
		fn serialize_i16(self, i16);
		fn serialize_i32(self, i32);
		fn serialize_i64(self, i64);
		fn serialize_i128(self, i128);
		fn serialize_u8(self, u8);
		fn serialize_u16(self, u16);
		fn serialize_u32(self, u32);
		fn serialize_u64(self, u64);
		fn serialize_u128(self, u128);
		fn serialize_f32(self, f32);
		fn serialize_f64(self, f64);
		fn serialize_char(self, char);
		fn serialize_str(self, &str);
		fn serialize_bytes(self, &[u8]);
		fn serialize_none(self);
		fn serialize_some<T>(self, &T);
		fn serialize_unit(self);
		fn serialize_unit_variant(self, &'static str, u32, &'static str);
		fn serialize_newtype_variant<T>(self, &'static str, u32, &'static str, &T);
	}

	serialize_type_error! {
		fn serialize_seq(self, Option<usize>) -> Result<Impossible>;
		fn serialize_tuple(self, usize) -> Result<Impossible>;
		fn serialize_tuple_variant(self, &'static str, u32, &'static str, usize) -> Result<Impossible>;
		fn serialize_map(self, Option<usize>) -> Result<Impossible>;
		fn serialize_struct_variant(self, &'static str, u32, &'static str, usize) -> Result<Impossible>;
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
		self.begin_sexpr(name);
		self.end_sexpr();
		Ok(())
	}

	fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		self.begin_sexpr(name);
		value.serialize(Field {
			ser: &mut *self,
			name: None
		})?;
		self.end_sexpr();
		Ok(())
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		_len: usize
	) -> Result<Self> {
		self.begin_sexpr(name);
		Ok(self)
	}

	fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self> {
		self.begin_sexpr(name);
		Ok(self)
	}
}

impl<'a> SerializeTupleStruct for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		value.serialize(Field {
			ser: &mut **self,
			name: None
		})
	}

	fn end(self) -> Result<()> {
		self.end_sexpr();
		Ok(())
	}
}

impl<'a> SerializeStruct for &'a mut Serializer {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		// TODO this should probably not be self
		value.serialize(Field {
			ser: &mut **self,
			name: Some(key)
		})
	}

	fn end(self) -> Result<()> {
		self.end_sexpr();
		Ok(())
	}
}

/// This serializer will serialize all fields. It needs the field name for booleans and
/// sequences.
struct Field<'a> {
	ser: &'a mut Serializer,
	name: Option<&'static str>
}

macro_rules! serialize_integer {
	($($integer:ty)+) => {
		$(
			paste! {
				fn [<serialize_ $integer>](self, v: $integer) -> Result<()> {
					self.ser.write_integer(v);
					Ok(())
				}
			}
		)+
	};
}

impl<'a> ser::Serializer for Field<'a> {
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Sequence<'a>;
	type SerializeTuple = Sequence<'a>;
	type SerializeTupleStruct = &'a mut Serializer;
	type SerializeTupleVariant = Impossible;
	type SerializeMap = Impossible;
	type SerializeStruct = &'a mut Serializer;
	type SerializeStructVariant = Impossible;

	serialize_type_error! {
		fn serialize_char(self, char) = Error::Char;
		fn serialize_bytes(self, &[u8]) = Error::Bytes;
		fn serialize_unit(self) = Error::Unit;
		fn serialize_newtype_variant<T>(self, &'static str, u32, &'static str, &T) = Error::ComplexEnum;
	}

	serialize_type_error! {
		fn serialize_tuple_variant(self, &'static str, u32, &'static str, usize) -> Result<Impossible> = Error::ComplexEnum;
		fn serialize_map(self, Option<usize>) -> Result<Impossible> = Error::Map;
		fn serialize_struct_variant(self, &'static str, u32, &'static str, usize) -> Result<Impossible> = Error::ComplexEnum;
	}

	fn serialize_bool(self, v: bool) -> Result<()> {
		let name = self.name.ok_or(Error::UnnamedBoolean)?;
		if v {
			self.serialize_str(name)?;
		}
		Ok(())
	}

	serialize_integer! {
		i8 i16 i32 i64 i128 u8 u16 u32 u64 u128
	}

	fn serialize_f32(self, v: f32) -> Result<()> {
		self.ser.write_float(v);
		Ok(())
	}

	fn serialize_f64(self, v: f64) -> Result<()> {
		self.ser.write_float(v);
		Ok(())
	}

	fn serialize_str(self, v: &str) -> Result<()> {
		self.ser.write_str(v, true);
		Ok(())
	}

	fn serialize_none(self) -> Result<()> {
		Ok(())
	}

	fn serialize_some<T>(self, v: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		v.serialize(self)
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
		self.ser.serialize_unit_struct(name)
	}

	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str
	) -> Result<()> {
		self.ser.write_str(variant, false);
		Ok(())
	}

	fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		self.ser.serialize_newtype_struct(name, value)
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<Sequence<'a>> {
		let name = self.name.ok_or(Error::UnnamedSeq)?;
		let close_sexpr = match name {
			"" => false,
			name => {
				self.ser.begin_sexpr(name);
				true
			}
		};
		Ok(Sequence::new(self.ser, close_sexpr))
	}

	fn serialize_tuple(self, len: usize) -> Result<Sequence<'a>> {
		self.serialize_seq(Some(len))
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		len: usize
	) -> Result<&'a mut Serializer> {
		self.ser.serialize_tuple_struct(name, len)
	}

	fn serialize_struct(
		self,
		name: &'static str,
		len: usize
	) -> Result<&'a mut Serializer> {
		self.ser.serialize_struct(name, len)
	}
}

/// A sequence / tuple serializer that optionally closes an s-expr afterwards
struct Sequence<'a> {
	ser: &'a mut Serializer,
	close_sexpr: bool
}

impl<'a> Sequence<'a> {
	fn new(ser: &'a mut Serializer, close_sexpr: bool) -> Self {
		Self { ser, close_sexpr }
	}
}

impl<'a> SerializeSeq for Sequence<'a> {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		value.serialize(Field {
			ser: &mut *self.ser,
			name: None
		})
	}

	fn end(self) -> Result<()> {
		if self.close_sexpr {
			self.ser.end_sexpr();
		}
		Ok(())
	}
}

impl<'a> SerializeTuple for Sequence<'a> {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize
	{
		SerializeSeq::serialize_element(self, value)
	}

	fn end(self) -> Result<()> {
		SerializeSeq::end(self)
	}
}
