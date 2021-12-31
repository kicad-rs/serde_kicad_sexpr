#[macro_export]
macro_rules! untagged {
	(
		$(#[$attr:meta])*
		$vis:vis enum $name:ident {
			$(
				$(#[$variant_attr:meta])*
				$variant:ident($inner:ty)
			),+
		}
	) => {
		$(#[$attr])*
		#[derive(Serialize)]
		#[serde(untagged)]
		$vis enum $name {
			$(
				$(#[$variant_attr])*
				$variant($inner)
			),+
		}

		impl<'de> ::serde::Deserialize<'de> for $name
		where
			$($inner: ::serde::Deserialize<'de>),*
		{
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: ::serde::Deserializer<'de>
			{
				static VARIANTS: $crate::private::SyncLazy<
					::std::result::Result<
						[&'static str; $crate::count!($($variant)+)],
						::std::string::String
					>
				> = $crate::private::SyncLazy::new(|| ::std::result::Result::Ok([$({
					let extraction = <$inner as ::serde::Deserialize>::deserialize(
						$crate::private::NameExtractor
					).unwrap_err();
					match extraction {
						$crate::private::Extraction::Ok(name) => name,
						$crate::private::Extraction::Err(err) => return Err(err)
					}
				}),+]));
				let variants: &'static [&'static str] = VARIANTS
					.as_ref()
					.map_err(|err| <D::Error as ::serde::de::Error>::custom(err))?;

				struct Visitor(&'static [&'static str]);

				impl<'de> ::serde::de::Visitor<'de> for Visitor {
					type Value = $name;

					fn expecting(
						&self, f: &mut ::std::fmt::Formatter<'_>
					) -> ::std::fmt::Result {
						::std::fmt::Display::fmt(&::std::format_args!(
							"any s-expr with a name in {:?}",
							self.0
						), f)
					}

					fn visit_enum<A>(self, data: A) -> ::std::result::Result<$name, A::Error>
					where
						A: ::serde::de::EnumAccess<'de>
					{
						let (variant_name, variant): (::std::borrow::Cow<'de, str>, _) =
							data.variant()?;

						let mut i = 0;
						$(
							if variant_name == self.0[i] {
								let inner: $inner =
									::serde::de::VariantAccess::newtype_variant(variant)?;
								return ::std::result::Result::Ok($name::$variant(inner));
							}
							i += 1;
						)+
						let _ = i;

						return ::std::result::Result::Err(
							<A::Error as ::serde::de::Error>::invalid_value(
								::serde::de::Unexpected::Other(&variant_name),
								&self
							)
						);
					}
				}

				deserializer.deserialize_enum(
					stringify!($name),
					variants,
					Visitor(variants)
				)
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! count {
	() => {
		0
	};

	($x:ident $($xs:ident)*) => {
		1 + count!($($xs)*)
	}
}

#[cfg(test)]
mod tests {
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Deserialize, PartialEq, Serialize)]
	#[serde(deny_unknown_fields, rename = "foo")]
	struct Foo;

	#[derive(Debug, Deserialize, PartialEq, Serialize)]
	#[serde(deny_unknown_fields, rename = "bar")]
	struct Bar;

	untagged! {
		#[derive(Debug, PartialEq)]
		enum FooOrBar {
			Foo(Foo),
			Bar(Bar)
		}
	}

	#[test]
	fn deserialize_foo() {
		let input = "(foo)";
		let expected = FooOrBar::Foo(Foo);

		let parsed: FooOrBar =
			crate::from_str(input).expect("Failed to parse input");
		assert_eq!(parsed, expected);
	}

	#[test]
	fn deserialize_bar() {
		let input = "(bar)";
		let expected = FooOrBar::Bar(Bar);

		let parsed: FooOrBar =
			crate::from_str(input).expect("Failed to parse input");
		assert_eq!(parsed, expected);
	}
}
