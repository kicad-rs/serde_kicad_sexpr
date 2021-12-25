use paste::paste;
use serde::{
	de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
	forward_to_deserialize_any, Deserialize
};
use std::{borrow::Cow, fmt::Display, str::FromStr};

mod error;
pub use error::Error;

pub struct Deserializer<'de> {
	input: &'de str
}

impl<'de> Deserializer<'de> {
	pub fn from_str(input: &'de str) -> Self {
		Self { input }
	}
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn from_str<'de, T>(input: &'de str) -> Result<T>
where
	T: Deserialize<'de>
{
	let mut deserializer = Deserializer::from_str(input);
	T::deserialize(&mut deserializer)
}

impl<'de> Deserializer<'de> {
	fn skip_whitespace(&mut self) {
		self.input = self.input.trim_start();
	}

	fn peek_char(&mut self) -> Result<char> {
		self.input.chars().next().ok_or(Error::Eof)
	}

	fn next_char(&mut self) -> Result<char> {
		let ch = self.peek_char()?;
		self.input = &self.input[ch.len_utf8()..];
		Ok(ch)
	}

	fn peek_identifier(&mut self) -> Option<&'de str> {
		let len: usize = self
			.input
			.chars()
			.take_while(|ch| ch.is_ascii_alphabetic() || *ch == '_')
			.map(|ch| ch.len_utf8())
			.sum();
		if len == 0 {
			return None;
		}
		Some(&self.input[..len])
	}

	fn peek_sexpr_identifier(&mut self) -> Result<&'de str> {
		let mut chars = self.input.chars();
		if chars.next().ok_or(Error::Eof)? != '(' {
			return Err(Error::ExpectedSExpr);
		}
		let paren = '('.len_utf8();
		let len: usize = chars
			.take_while(|ch| ch.is_ascii_alphabetic() || *ch == '_')
			.map(|ch| ch.len_utf8())
			.sum();
		if len == 0 {
			return Err(Error::ExpectedIdentifier);
		}
		Ok(&self.input[paren..paren + len])
	}

	fn consume(&mut self, len: usize) -> Result<()> {
		if self.input.len() < len {
			return Err(Error::Eof);
		}
		self.input = &self.input[len..];
		Ok(())
	}

	fn parse_number<T>(&mut self) -> Result<T>
	where
		T: FromStr,
		T::Err: Display
	{
		let len = self
			.input
			.chars()
			.take_while(|ch| ch.is_ascii_digit() || *ch == '-' || *ch == '.')
			.map(|ch| ch.len_utf8())
			.sum();
		if len == 0 {
			return Err(Error::ExpectedNumber);
		}
		let number = &self.input[..len];
		let number = number
			.parse()
			.map_err(|err: T::Err| Error::Message(err.to_string()))?;
		self.input = &self.input[len..];
		Ok(number)
	}

	fn parse_string(&mut self) -> Result<Cow<'de, str>> {
		match self.peek_char()? {
			'(' => Err(Error::ExpectedString),

			'"' => {
				self.consume('"'.len_utf8())?;
				let mut value = String::new();
				loop {
					let len: usize = self
						.input
						.chars()
						.take_while(|ch| *ch != '"')
						.map(|ch| ch.len_utf8())
						.sum();
					if len >= self.input.len() {
						return Err(Error::Eof);
					}

					let mut start_idx = value.chars().count();
					value += &self.input[..len + 1];
					self.input = &self.input[len + 1..];
					while let Some(idx) = (&value[start_idx..]).find(r"\\") {
						let idx = start_idx + idx;
						value.replace_range(idx..idx + 2, r"\");
						start_idx = idx + 1;
					}

					if value.ends_with(r#"\""#) && start_idx < value.len() - 1 {
						value.remove(value.len() - 2);
					} else if value.ends_with(r#"""#) {
						value.remove(value.len() - 1);
						break;
					} else {
						unreachable!();
					}
				}
				Ok(value.into())
			},

			_ => {
				let len = self
					.input
					.chars()
					.take_while(|ch| !ch.is_ascii_whitespace() && *ch != ')')
					.map(|ch| ch.len_utf8())
					.sum();
				if len == 0 {
					return Err(Error::Eof);
				}
				let value = &self.input[..len];
				self.input = &self.input[len..];
				Ok(value.into())
			}
		}
	}
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, _: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		return Err(Error::ExpectedStruct);
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_map(SExpr::new(self, name, fields)?)
	}

	fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		SExpr::consume_beginning(self, name)?;
		if self.next_char()? != ')' {
			return Err(Error::ExpectedEoe);
		}
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_seq(SExprTuple::new(self, name)?)
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		_len: usize,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_seq(SExprTuple::new(self, name)?)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit seq tuple map enum identifier ignored_any
	}
}

/// Deserialise an s-expr.
struct SExpr<'a, 'de> {
	de: &'a mut Deserializer<'de>,
	fields: &'static [&'static str],
	index: usize,
	skip_to: Option<usize>
}

impl<'a, 'de> SExpr<'a, 'de> {
	fn consume_beginning(de: &mut Deserializer<'de>, name: &'static str) -> Result<()> {
		de.skip_whitespace();
		if de.peek_sexpr_identifier()? != name {
			return Err(Error::ExpectedSExprIdentifier(name));
		}
		de.consume(name.len() + '('.len_utf8())?;
		Ok(())
	}

	fn new(
		de: &'a mut Deserializer<'de>,
		name: &'static str,
		fields: &'static [&'static str]
	) -> Result<Self> {
		Self::consume_beginning(de, name)?;
		Ok(Self {
			de,
			fields,
			index: 0,
			skip_to: None
		})
	}
}

impl<'a, 'de> MapAccess<'de> for SExpr<'a, 'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
	where
		K: DeserializeSeed<'de>
	{
		self.de.skip_whitespace();
		if self.skip_to.is_none() && self.de.peek_char()? == ')' {
			self.de.consume(1)?;
			// technically we're done, but there could be booleans that are false, so we'll
			// deserialize those as None/false eventhough they don't exist in the input.
			self.skip_to = Some(self.index + 1);
		}

		if self.index >= self.fields.len() {
			return Ok(None);
		}
		seed.deserialize(FieldIdent(self.fields[self.index]))
			.map(Some)
	}

	fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
	where
		T: DeserializeSeed<'de>
	{
		if self.index >= self.fields.len() {
			panic!("There was no key and there is no value");
		}

		// booleans are represented in this weird way where they are simply missing if they are
		// false. This means that if we detect a boolean ahead of our current index, we'll assume
		// everything inbetween is either None or false, and skip ahead.
		if let Some(skip_to) = self.skip_to {
			if skip_to == self.index {
				self.skip_to = None;
				self.index += 1;
				return seed.deserialize(TrueField);
			}
			self.index += 1;
			return seed.deserialize(MissingField);
		}
		if let Some(identifier) = self.de.peek_identifier() {
			if self.fields[self.index] == identifier {
				self.index += 1;
				return seed.deserialize(TrueField);
			}
			for i in self.index + 1..self.fields.len() {
				if self.fields[i] == identifier {
					self.skip_to = Some(i);
					self.index += 1;
					return seed.deserialize(MissingField);
				}
			}
		}

		self.index += 1;
		seed.deserialize(Field::new(self.de))
	}
}

/// Deserialize an s-expr in tuple format. It cannot contain booleans.
struct SExprTuple<'a, 'de> {
	de: &'a mut Deserializer<'de>
}

impl<'a, 'de> SExprTuple<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>, name: &'static str) -> Result<Self> {
		SExpr::consume_beginning(de, name)?;
		Ok(Self { de })
	}
}

impl<'a, 'de> SeqAccess<'de> for SExprTuple<'a, 'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: DeserializeSeed<'de>
	{
		// we can skip the boolean handling logic of SExpr as we don't have any identifiers
		// to use
		self.de.skip_whitespace();
		if self.de.peek_char()? == ')' {
			return Ok(None);
		}
		seed.deserialize(Field::new(self.de)).map(Some)
	}
}

/// Deserialize a field's ident.
struct FieldIdent(&'static str);

impl<'de> de::Deserializer<'de> for FieldIdent {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_borrowed_str(self.0)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map struct enum identifier ignored_any
	}
}

/// Deserialize a single boolean with value `true`.
struct TrueField;

impl<'de> de::Deserializer<'de> for TrueField {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_bool(true)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map struct enum identifier ignored_any
	}
}

/// Deserialize either a boolean with value `false` or an option with value `None`.
struct MissingField;

impl<'de> de::Deserializer<'de> for MissingField {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_none()
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_bool(false)
	}

	forward_to_deserialize_any! {
		i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map struct enum identifier ignored_any
	}
}

/// A non-boolean field.
struct Field<'a, 'de> {
	de: &'a mut Deserializer<'de>
}

impl<'a, 'de> Field<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>) -> Self {
		Self { de }
	}
}

macro_rules! forward_to_parse_number {
	($($ident:ident)+) => {
		$(
			paste! {
				fn [<deserialize_ $ident>]<V>(self, visitor: V) -> Result<V::Value>
				where
					V: Visitor<'de>
				{
					visitor.[<visit_ $ident>](self.de.parse_number()?)
				}
			}
		)+
	};
}

impl<'a, 'de> de::Deserializer<'de> for Field<'a, 'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		match self.de.peek_char()? {
			ch @ '0'..='9' | ch @ '-' | ch @ '.' => match self.de.input.find('.') {
				Some(idx)
					if (&self.de.input[..idx]).contains(|ch: char| ch.is_ascii_whitespace()) =>
				{
					if ch == '-' {
						self.deserialize_i64(visitor)
					} else {
						self.deserialize_u64(visitor)
					}
				},
				_ => self.deserialize_f32(visitor)
			},
			'(' => Err(Error::MissingSExprInfo),
			_ => self.deserialize_string(visitor)
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		self.deserialize_string(visitor)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let value = self.de.parse_string()?;
		match value {
			Cow::Borrowed(value) => visitor.visit_borrowed_str(value),
			Cow::Owned(value) => visitor.visit_string(value)
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		// If we arrived at this point in the code, there's no way for the option to be None
		visitor.visit_some(self)
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_map(SExpr::new(self.de, name, fields)?)
	}

	fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		SExpr::consume_beginning(self.de, name)?;
		if self.de.next_char()? != ')' {
			return Err(Error::ExpectedEoe);
		}
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_seq(SExprTuple::new(self.de, name)?)
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		_len: usize,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_seq(SExprTuple::new(self.de, name)?)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_enum(self)
	}

	forward_to_parse_number! {
		i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
	}

	forward_to_deserialize_any! {
		bool char bytes byte_buf unit seq tuple map identifier ignored_any
	}
}

impl<'a, 'de> EnumAccess<'de> for Field<'a, 'de> {
	type Error = Error;
	type Variant = UnitVariant;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where
		V: DeserializeSeed<'de>
	{
		Ok((seed.deserialize(self)?, UnitVariant))
	}
}

/// This will deserialize only unit variants.
struct UnitVariant;

impl<'de> VariantAccess<'de> for UnitVariant {
	type Error = Error;

	fn unit_variant(self) -> Result<()> {
		Ok(())
	}

	fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
	where
		T: DeserializeSeed<'de>
	{
		Err(Error::NonUnitEnumVariant)
	}

	fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		Err(Error::NonUnitEnumVariant)
	}

	fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		Err(Error::NonUnitEnumVariant)
	}
}
