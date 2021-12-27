use serde::{
	de::{
		self, Deserialize, Deserializer, EnumAccess, MapAccess, SeqAccess, Visitor
	},
	ser::{Serialize, Serializer}
};
use std::{
	fmt::{self, Formatter},
	marker::PhantomData
};

/// Deserialize an [`Option`] in a way that is supported by the s-expression format.
///
/// ### Example
///
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "size")]
/// struct Size(f32, f32);
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "thickness")]
/// struct Thickness(f32);
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "font")]
/// struct Font {
/// 	size: Size,
///
/// 	// This attribute enables our custom deserialize logic.
/// 	#[serde(with = "serde_sexpr::Option")]
/// 	thickness: Option<Thickness>,
///
/// 	bold: bool
/// }
/// # assert_eq!(
/// # 	serde_sexpr::from_str::<Font>("(font (size 1 1) bold)").unwrap(),
/// # 	Font { size: Size(1.0, 1.0), thickness: None, bold: true }
/// # );
/// ```
///
/// ### Description
///
/// For a more detailed description, see [`deserialize_option`].
pub struct OptionDef<T>(PhantomData<T>);

impl<'de, T> OptionDef<T>
where
	T: Deserialize<'de>
{
	pub fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
	where
		D: Deserializer<'de>
	{
		deserialize_option(deserializer)
	}
}

impl<T> OptionDef<T>
where
	Option<T>: Serialize
{
	pub fn serialize<S>(this: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer
	{
		this.serialize(serializer)
	}
}

/// Deserialize an [`Option`] in a way that is supported by the s-expression format.
///
/// ### Example
///
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "size")]
/// struct Size(f32, f32);
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "thickness")]
/// struct Thickness(f32);
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Deserialize, Serialize)]
/// #[serde(deny_unknown_fields, rename = "font")]
/// struct Font {
/// 	size: Size,
///
/// 	// This attribute enables our custom deserialize logic.
/// 	#[serde(deserialize_with = "serde_sexpr::deserialize_option")]
/// 	thickness: Option<Thickness>,
///
/// 	bold: bool
/// }
/// # assert_eq!(
/// # 	serde_sexpr::from_str::<Font>("(font (size 1 1) bold)").unwrap(),
/// # 	Font { size: Size(1.0, 1.0), thickness: None, bold: true }
/// # );
/// ```
///
/// ### Description
///
/// The s-expression format is not only not self-describing, but also does not provide any way
/// to see if a value is "missing" (i.e. [`None`]) without knowing its type. Unfortunately, serde
/// expects us to decide if the value is present before we know its type: In the above example,
/// we have the input string `"bold)"` and need to know if `thickness` is present or not, without
/// knowing that `thickness` is an s-expr and not, say, an enum that has a variant called `bold`.
///
/// This custom deserialize logic therefore avoids calling [`Deserializer::deserialize_option`]
/// alltogether. Instead, we'll try to deserialize the value as if it was present, and return
/// [`None`] if the deserializer returns an error before calling the visitor. This is likely
/// indicative of a type error, that would indicate a missing value.
///
/// ### Drawbacks
///
/// Using this deserialize logic might hide errors in the input. If this optional value
/// is the last value that gets deserialized, and the deserialization failed due to some
/// error other than a type error, it might get hidden.
///
/// Also, if trying to deserialize the value alters the state of the deserializer, it could
/// lead to incorrect deserialization.
pub fn deserialize_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>
{
	// this flag will be set if any visitor method was called
	let mut flag = false;

	// try to deserialize a present value
	let result = T::deserialize(OptionDeserializer {
		de: deserializer,
		flag: &mut flag
	});

	// if the flag is not set and we don't have a value, assume a non-present value
	match result {
		Ok(value) => Ok(Some(value)),
		Err(_) if !flag => Ok(None),
		Err(err) => Err(err)
	}
}

/// A deserializer that intercepts the visitor with our custom visitor.
struct OptionDeserializer<'a, D> {
	de: D,
	flag: &'a mut bool
}

macro_rules! forward_deserializer {
	($(fn $ident:ident <$visitor:ident>(
		self,
		$visitor_arg:ident : $visitor_arg_ty:ty
		$(, $arg:ident : $arg_ty:ty)*
	);)+) => {
		$(
			fn $ident<$visitor>(
				self,
				$($arg: $arg_ty,)*
				$visitor_arg: $visitor_arg_ty
			) -> Result<$visitor::Value, Self::Error>
			where
				$visitor: Visitor<'de>
			{
				let $visitor_arg = OptionVisitor {
					visitor: $visitor_arg,
					flag: self.flag
				};
				self.de.$ident($($arg,)* $visitor_arg)
			}
		)+
	};
}

impl<'a, 'de, D> Deserializer<'de> for OptionDeserializer<'a, D>
where
	D: Deserializer<'de>
{
	type Error = D::Error;

	forward_deserializer! {
		fn deserialize_any<V>(self, visitor: V);
		fn deserialize_bool<V>(self, visitor: V);
		fn deserialize_i8<V>(self, visitor: V);
		fn deserialize_i16<V>(self, visitor: V);
		fn deserialize_i32<V>(self, visitor: V);
		fn deserialize_i64<V>(self, visitor: V);
		fn deserialize_i128<V>(self, visitor: V);
		fn deserialize_u8<V>(self, visitor: V);
		fn deserialize_u16<V>(self, visitor: V);
		fn deserialize_u32<V>(self, visitor: V);
		fn deserialize_u64<V>(self, visitor: V);
		fn deserialize_u128<V>(self, visitor: V);
		fn deserialize_f32<V>(self, visitor: V);
		fn deserialize_f64<V>(self, visitor: V);
		fn deserialize_char<V>(self, visitor: V);
		fn deserialize_str<V>(self, visitor: V);
		fn deserialize_string<V>(self, visitor: V);
		fn deserialize_bytes<V>(self, visitor: V);
		fn deserialize_byte_buf<V>(self, visitor: V);
		fn deserialize_option<V>(self, visitor: V);
		fn deserialize_unit<V>(self, visitor: V);
		fn deserialize_unit_struct<V>(self, visitor: V, name: &'static str);
		fn deserialize_newtype_struct<V>(self, visitor: V, name: &'static str);
		fn deserialize_seq<V>(self, visitor: V);
		fn deserialize_tuple<V>(self, visitor: V, len: usize);
		fn deserialize_tuple_struct<V>(self, visitor: V, name: &'static str, len: usize);
		fn deserialize_map<V>(self, visitor: V);
		fn deserialize_struct<V>(self, visitor: V, name: &'static str, fields: &'static [&'static str]);
		fn deserialize_enum<V>(self, visitor: V, name: &'static str, variants: &'static [&'static str]);
		fn deserialize_identifier<V>(self, visitor: V);
		fn deserialize_ignored_any<V>(self, visitor: V);
	}

	fn is_human_readable(&self) -> bool {
		self.de.is_human_readable()
	}
}

/// A visitor that will set a flag if any visit method was called (except for visit_none).
/// This indicates that the value was indeed present, i.e. any error return was not a
/// general type error but instead a problem deserializing the correct type.
struct OptionVisitor<'a, V> {
	visitor: V,
	flag: &'a mut bool
}

macro_rules! forward_visitor {
	($(fn $ident:ident <$error:ident>(self $(, $arg:ident : $arg_ty:ty)*);)+) => {
		$(
			fn $ident<$error>(self $(, $arg: $arg_ty,)*) -> Result<Self::Value, $error>
			where
				$error: de::Error
			{
				*self.flag = true;
				self.visitor.$ident($($arg),*)
			}
		)+
	};

	($(fn $ident:ident <$access:ident : $access_bound:path>(
		self $(, $arg:ident : $arg_ty:ty)*
	);)+) => {
		$(
			fn $ident<$access>(self $(, $arg: $arg_ty,)*) -> Result<Self::Value, $access::Error>
			where
				$access: $access_bound
			{
				*self.flag = true;
				self.visitor.$ident($($arg),*)
			}
		)+
	};
}

impl<'a, 'de, V> Visitor<'de> for OptionVisitor<'a, V>
where
	V: Visitor<'de>
{
	type Value = V::Value;

	fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str("optional ")?;
		self.visitor.expecting(f)
	}

	forward_visitor! {
		fn visit_bool<E>(self, v: bool);
		fn visit_i8<E>(self, v: i8);
		fn visit_i16<E>(self, v: i16);
		fn visit_i32<E>(self, v: i32);
		fn visit_i64<E>(self, v: i64);
		fn visit_i128<E>(self, v: i128);
		fn visit_u8<E>(self, v: u8);
		fn visit_u16<E>(self, v: u16);
		fn visit_u32<E>(self, v: u32);
		fn visit_u64<E>(self, v: u64);
		fn visit_u128<E>(self, v: u128);
		fn visit_f32<E>(self, v: f32);
		fn visit_f64<E>(self, v: f64);
		fn visit_char<E>(self, v: char);
		fn visit_str<E>(self, v: &str);
		fn visit_borrowed_str<E>(self, v: &'de str);
		fn visit_string<E>(self, v: String);
		fn visit_bytes<E>(self, v: &[u8]);
		fn visit_borrowed_bytes<E>(self, v: &'de [u8]);
		fn visit_byte_buf<E>(self, v: Vec<u8>);
		fn visit_unit<E>(self);
	}

	fn visit_none<E>(self) -> Result<Self::Value, E>
	where
		E: de::Error
	{
		// special case - if we don't set the flag and return an error, we'll
		// get None
		Err(E::custom(""))
	}

	forward_visitor! {
		fn visit_some<D: Deserializer<'de>>(self, de: D);
		fn visit_newtype_struct<D: Deserializer<'de>>(self, de: D);
		fn visit_seq<A: SeqAccess<'de>>(self, seq: A);
		fn visit_map<A: MapAccess<'de>>(self, map: A);
		fn visit_enum<A: EnumAccess<'de>>(self, data: A);
	}
}
