use serde::{
	de::{self, Deserializer, Visitor},
	forward_to_deserialize_any
};
use std::{
	error::Error,
	fmt::{self, Debug, Display, Formatter}
};

pub use once_cell::sync::Lazy as SyncLazy;

pub struct NameExtractor;

#[derive(Debug)]
pub enum Extraction {
	Ok(&'static str),
	Err(String)
}

impl Display for Extraction {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Ok(ok) => Display::fmt(ok, f),
			Self::Err(err) => Display::fmt(err, f)
		}
	}
}

impl Error for Extraction {}

impl de::Error for Extraction {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Err(msg.to_string())
	}
}

impl<'de> Deserializer<'de> for NameExtractor {
	type Error = Extraction;

	fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Extraction>
	where
		V: Visitor<'de>
	{
		return Err(Extraction::Err(
			"Expected to deserialize a struct".to_owned()
		));
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		_visitor: V
	) -> Result<V::Value, Extraction>
	where
		V: Visitor<'de>
	{
		return Result::Err(Extraction::Ok(name));
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		_visitor: V
	) -> Result<V::Value, Extraction>
	where
		V: Visitor<'de>
	{
		return Result::Err(Extraction::Ok(name));
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		_len: usize,
		_visitor: V
	) -> Result<V::Value, Extraction>
	where
		V: Visitor<'de>
	{
		return Result::Err(Extraction::Ok(name));
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		_fields: &'static [&'static str],
		_visitor: V
	) -> Result<V::Value, Extraction>
	where
		V: Visitor<'de>
	{
		return Result::Err(Extraction::Ok(name));
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes
		byte_buf option unit seq tuple map enum identifier ignored_any
	}
}
