use paste::paste;
use serde::{
	de::{
		self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess,
		Visitor
	},
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
	let value = T::deserialize(&mut deserializer)?;
	Ok(value)
}

enum Token {
	String,
	Int,
	Float,
	SExpr
}

impl<'de> Deserializer<'de> {
	fn check_no_trailing_tokens(&mut self) -> Result<()> {
		self.skip_whitespace();
		if !self.input.is_empty() {
			return Err(Error::TrailingTokens);
		}
		Ok(())
	}

	fn skip_whitespace(&mut self) {
		self.input = self.input.trim_start();
	}

	fn peek_char(&self) -> Result<char> {
		self.input.chars().next().ok_or(Error::Eof)
	}

	fn next_char(&mut self) -> Result<char> {
		let ch = self.peek_char()?;
		self.input = &self.input[ch.len_utf8()..];
		Ok(ch)
	}

	fn peek_token(&self) -> Result<Token> {
		let mut chars = self.input.chars().peekable();
		if chars.peek().is_none() {
			return Err(Error::Eof);
		}

		let mut int = true;
		while let Some(ch) = chars.next() {
			match ch {
				'(' => return Ok(Token::SExpr),
				'.' => {
					int = false;
				},
				'-' => {},
				ch if ch.is_ascii_whitespace() => break,
				ch if ch.is_ascii_digit() => {},
				_ => return Ok(Token::String)
			};
		}

		Ok(match int {
			true => Token::Int,
			false => Token::Float
		})
	}

	fn peek_identifier(&self) -> Option<&'de str> {
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

	fn peek_sexpr_identifier(&self) -> Result<&'de str> {
		let mut chars = self.input.chars();
		let next = chars.next().ok_or(Error::Eof)?;
		if next != '(' {
			return Err(Error::ExpectedSExpr(next));
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
			.take_while(|ch| !ch.is_ascii_whitespace() && *ch != ')')
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
		let v = visitor.visit_map(SExpr::new(self, name, fields)?)?;
		self.check_no_trailing_tokens()?;
		Ok(v)
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		SExpr::consume_beginning(self, name)?;
		if self.next_char()? != ')' {
			return Err(Error::ExpectedEoe);
		}
		self.check_no_trailing_tokens()?;
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let v = visitor.visit_seq(SExprTuple::new(self, name)?)?;
		self.check_no_trailing_tokens()?;
		Ok(v)
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
		let v = visitor.visit_seq(SExprTuple::new(self, name)?)?;
		self.check_no_trailing_tokens()?;
		Ok(v)
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
		let v = visitor.visit_enum(Enum::new(self))?;
		self.check_no_trailing_tokens()?;
		Ok(v)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit seq tuple map identifier ignored_any
	}
}

/// Deserialize an enum with only newtype variants whose variant names match the
/// names of the contained s-exprs.
struct Enum<'a, 'de> {
	de: &'a mut Deserializer<'de>
}

impl<'a, 'de> Enum<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>) -> Self {
		Self { de }
	}
}

impl<'a, 'de> EnumAccess<'de> for Enum<'a, 'de> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where
		V: DeserializeSeed<'de>
	{
		Ok((
			seed.deserialize(FieldIdent(self.de.peek_sexpr_identifier()?))?,
			self
		))
	}
}

impl<'a, 'de> VariantAccess<'de> for Enum<'a, 'de> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		Err(Error::NonNewtypeEnumVariant)
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>
	{
		seed.deserialize(self.de)
	}

	fn tuple_variant<V>(
		self,
		_len: usize,
		_visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		Err(Error::NonNewtypeEnumVariant)
	}

	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		_visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		Err(Error::NonNewtypeEnumVariant)
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
	fn consume_beginning(
		de: &mut Deserializer<'de>,
		name: &'static str
	) -> Result<()> {
		de.skip_whitespace();
		let peek = de.peek_sexpr_identifier()?;
		if peek != name {
			return Err(Error::ExpectedSExprIdentifier(name, peek.to_owned()));
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

	fn check_eoe(&mut self) -> Result<()> {
		self.de.skip_whitespace();
		if self.skip_to.is_none() && self.de.peek_char()? == ')' {
			self.de.consume(1)?;
			// technically we're done, but there could be booleans that are false, so we'll
			// deserialize those as None/false eventhough they don't exist in the input.
			self.skip_to = Some(self.fields.len() + 1);
		}
		Ok(())
	}

	fn next_value_seed_impl<T>(&mut self, seed: T) -> Result<T::Value>
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
				return seed.deserialize(TrueField);
			}
			return seed.deserialize(MissingField);
		}
		if let Some(identifier) = self.de.peek_identifier() {
			if self.fields[self.index] == identifier {
				self.de.consume(identifier.len())?;
				return seed.deserialize(TrueField);
			}
			for i in self.index + 1..self.fields.len() {
				if self.fields[i] == identifier {
					self.de.consume(identifier.len())?;
					self.skip_to = Some(i);
					return seed.deserialize(MissingField);
				}
			}
		}

		seed.deserialize(Field::new(self.de, Some(self.fields[self.index])))
	}
}

impl<'a, 'de> MapAccess<'de> for SExpr<'a, 'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
	where
		K: DeserializeSeed<'de>
	{
		self.check_eoe()?;

		loop {
			if self.index >= self.fields.len() {
				return Ok(None);
			}

			// special case: if the ident is empty ("") and we are set to skip it, don't even
			// return the field.
			if self.fields[self.index] == "" {
				if let Some(skip_to) = self.skip_to {
					if skip_to == self.index {
						self.skip_to = None;
					}
					self.index += 1;
					continue;
				}
			}

			break;
		}

		seed.deserialize(FieldIdent(self.fields[self.index]))
			.map(Some)
	}

	fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
	where
		T: DeserializeSeed<'de>
	{
		let value = self.next_value_seed_impl(seed)?;
		self.index += 1;
		self.check_eoe()?;
		Ok(value)
	}
}

/// Deserialize an s-expr in tuple format. It cannot contain booleans.
struct SExprTuple<'a, 'de> {
	de: &'a mut Deserializer<'de>,
	end: bool
}

impl<'a, 'de> SExprTuple<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>, name: &'static str) -> Result<Self> {
		SExpr::consume_beginning(de, name)?;
		Ok(Self { de, end: false })
	}

	fn check_eoe(&mut self) -> Result<()> {
		if self.end {
			return Ok(());
		}

		self.de.skip_whitespace();
		if self.de.peek_char()? == ')' {
			self.de.consume(')'.len_utf8())?;
			self.end = true;
		}
		Ok(())
	}
}

impl<'a, 'de> SeqAccess<'de> for SExprTuple<'a, 'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: DeserializeSeed<'de>
	{
		self.check_eoe()?;
		if self.end {
			return Ok(None);
		}
		let value = seed.deserialize(Field::new(self.de, None))?;
		self.check_eoe()?;
		Ok(Some(value))
	}
}

/// Deserialize a field's ident.
struct FieldIdent<'a>(&'a str);

impl<'a, 'de> de::Deserializer<'de> for FieldIdent<'a>
where
	'a: 'de
{
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

/// A field whose value does not match its ident. This means that if a boolean gets requested,
/// we must return false without touching the input.
///
/// We still store the ident if we know it, so that we can parse a sequence like
/// (<ident> <values..>). The empty ident (`""`) is treated as a special case to consume
/// the remaining fields of the current expression.
struct Field<'a, 'de> {
	de: &'a mut Deserializer<'de>,
	ident: Option<&'static str>
}

impl<'a, 'de> Field<'a, 'de> {
	fn new(de: &'a mut Deserializer<'de>, ident: Option<&'static str>) -> Self {
		Self { de, ident }
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
		match self.de.peek_token()? {
			Token::Int if self.de.peek_char()? == '-' => {
				self.deserialize_i64(visitor)
			},
			Token::Int => self.deserialize_u64(visitor),
			Token::Float => self.deserialize_f64(visitor),
			Token::String => self.deserialize_string(visitor),
			Token::SExpr if Some(self.de.peek_sexpr_identifier()?) == self.ident => {
				self.deserialize_seq(visitor)
			},
			Token::SExpr => Err(Error::MissingSExprInfo(
				self.de.peek_sexpr_identifier()?.to_owned()
			))
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		visitor.visit_bool(false)
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

	fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		// we'll need to know the type of Some (i.e. the s-expr tag) to see if it is present in
		// the input or not
		// however, serde doesn't give us this type of information, so we'll just error
		return Err(Error::DeserializeOption);
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

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let ident = match self.ident {
			Some(ident) => ident,
			None => {
				return Err(Error::MissingSExprInfo(
					self.de.peek_sexpr_identifier()?.to_owned()
				));
			}
		};
		self.deserialize_unit_struct(ident, visitor)
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		SExpr::consume_beginning(self.de, name)?;
		if self.de.next_char()? != ')' {
			return Err(Error::ExpectedEoe);
		}
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		self.deserialize_tuple_struct(name, 1, visitor)
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

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let ident = match self.ident {
			Some(ident) => ident,
			None => {
				return Err(Error::MissingSExprInfo(
					self.de.peek_sexpr_identifier()?.to_owned()
				));
			}
		};
		match ident {
			"" => {
				// special case: we'll return the remaining tokens of the current s-expr
				visitor.visit_seq(self)
			},
			_ => visitor.visit_seq(SExprTuple::new(self.de, ident)?)
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		self.deserialize_seq(visitor)
	}

	forward_to_parse_number! {
		i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
	}

	forward_to_deserialize_any! {
		char bytes byte_buf map identifier ignored_any
	}
}

impl<'a, 'de> SeqAccess<'de> for Field<'a, 'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: DeserializeSeed<'de>
	{
		self.de.skip_whitespace();
		if self.de.peek_char()? == ')' {
			return Ok(None);
		}
		seed.deserialize(Field::new(self.de, None)).map(Some)
	}
}

impl<'a, 'de> EnumAccess<'de> for Field<'a, 'de> {
	type Error = Error;
	type Variant = Either<UnitVariant, NewtypeVariant<'a, 'de>>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where
		V: DeserializeSeed<'de>
	{
		Ok(match self.de.peek_token()? {
			Token::SExpr => {
				let str = self.de.peek_sexpr_identifier()?;
				(
					seed.deserialize(FieldIdent(str))?,
					Either::Right(NewtypeVariant { de: self.de })
				)
			},
			_ => (seed.deserialize(self)?, Either::Left(UnitVariant))
		})
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

	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		_visitor: V
	) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		Err(Error::NonUnitEnumVariant)
	}
}

/// This will deserialize only newtype variants.
struct NewtypeVariant<'a, 'de> {
	de: &'a mut Deserializer<'de>
}

impl<'a, 'de> VariantAccess<'de> for NewtypeVariant<'a, 'de> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		Err(Error::NonNewtypeEnumVariant)
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>
	{
		seed.deserialize(Field::new(self.de, None))
	}

	fn tuple_variant<V>(
		self,
		_len: usize,
		_visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		Err(Error::NonNewtypeEnumVariant)
	}

	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		_visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		Err(Error::NonNewtypeEnumVariant)
	}
}

/// An `Either` type for `VariantAccess`.
enum Either<L, R> {
	Left(L),
	Right(R)
}

impl<'de, L, R> VariantAccess<'de> for Either<L, R>
where
	L: VariantAccess<'de>,
	R: VariantAccess<'de, Error = L::Error>
{
	type Error = L::Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		match self {
			Self::Left(l) => l.unit_variant(),
			Self::Right(r) => r.unit_variant()
		}
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>
	{
		match self {
			Self::Left(l) => l.newtype_variant_seed(seed),
			Self::Right(r) => r.newtype_variant_seed(seed)
		}
	}

	fn tuple_variant<V>(
		self,
		len: usize,
		visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		match self {
			Self::Left(l) => l.tuple_variant(len, visitor),
			Self::Right(r) => r.tuple_variant(len, visitor)
		}
	}

	fn struct_variant<V>(
		self,
		fields: &'static [&'static str],
		visitor: V
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>
	{
		match self {
			Self::Left(l) => l.struct_variant(fields, visitor),
			Self::Right(r) => r.struct_variant(fields, visitor)
		}
	}
}
