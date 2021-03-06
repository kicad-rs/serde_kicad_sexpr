use serde::de;
use std::fmt::{self, Debug, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum ErrorKind {
	#[error("{0}")]
	Message(String),

	/// This error will be returned if you request to deserialize anything but a
	/// struct at root level.
	#[error("Expected to deserialize a struct at root level")]
	ExpectedStruct,

	/// This error will be returned if the input ends unexpectedly.
	#[error("Unexpected end of input")]
	Eof,

	/// This error will be returned if an opening s-expr was expected, but some
	/// other token was found.
	#[error("Expected s-expr, found token '{0}'")]
	ExpectedSExpr(char),

	/// This error will be returned if an opening s-expr with a certain name was
	/// expected, but some other token was found.
	#[error("Expected s-expr identifier {0}, found {1}")]
	ExpectedSExprIdentifier(&'static str, String),

	/// This error will be returned if the end of the s-expr was expected, but some
	/// other token was found.
	#[error("Expected end of expression")]
	ExpectedEoe,

	/// This error will be returned if an identifier was expected, but some other
	/// token was found.
	#[error("Expected identifier")]
	ExpectedIdentifier,

	/// This error will be returned if a number was expected, but some other token
	/// was found.
	#[error("Expected number")]
	ExpectedNumber,

	/// This error will be returned if a string was expected, but some other token
	/// was found.
	#[error("Expected string")]
	ExpectedString,

	/// This error will be returned if an option was requested. [`Option`] is
	/// **not supported** by this data format.
	#[error("std::option::Option cannot be deserialized by this data format")]
	DeserializeOption,

	/// This error will be returned if an s-expr is found, but its name (and fields)
	/// were not supplied to the deserializer (e.g. `deserialize_any` was called).
	#[error("Missing s-expr type info for {0}")]
	MissingSExprInfo(String),

	/// This error will be returned when attempting to deserialize a non-newtype
	/// enum variant in a tagged way.
	#[error("Non-newtype enum variants are not supported in tagged format")]
	NonNewtypeEnumVariant,

	/// This error will be returned when attempting to deserialize a non-unit enum
	/// variant in an untagged way.
	#[error("Non-unit enum variants are not supported in untagged format")]
	NonUnitEnumVariant,

	/// This error will be returned if there were trailing tokens after the deserialization
	/// finished.
	#[error("Trailing tokens")]
	TrailingTokens
}

#[non_exhaustive]
pub struct Error {
	pub kind: ErrorKind,

	#[cfg(feature = "backtrace")]
	pub(super) backtrace: once_cell::sync::Lazy<
		backtrace::Backtrace,
		Box<dyn FnOnce() -> backtrace::Backtrace>
	>
}

impl Error {
	#[cfg(feature = "backtrace")]
	pub fn backtrace(&self) -> &backtrace::Backtrace {
		&*self.backtrace
	}
}

impl Debug for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut dbg = f.debug_struct("Error");
		dbg.field("message", &self.kind.to_string());
		dbg.field("kind", &self.kind);
		#[cfg(feature = "backtrace")]
		dbg.field("backtrace", self.backtrace());
		dbg.finish()
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.kind, f)
	}
}

impl std::error::Error for Error {}

impl de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Error {
			kind: ErrorKind::Message(msg.to_string()),
			#[cfg(feature = "backtrace")]
			backtrace: {
				let bt = backtrace::Backtrace::new_unresolved();
				once_cell::sync::Lazy::new(Box::new(move || {
					let mut bt = bt;
					bt.resolve();
					bt
				}))
			}
		}
	}
}
