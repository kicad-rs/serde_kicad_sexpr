use serde::de;
use std::fmt::Display;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
	#[error("{0}")]
	Message(String),

	/// This error will be returned if you request to deserialize anything but a struct at root
	/// level
	#[error("Expected to deserialize a struct at root level")]
	ExpectedStruct,

	/// This error will be returned if the input ends unexpectedly.
	#[error("Unexpected end of input")]
	Eof,

	/// This error will be returned if an opening s-expr was expected, but some other token was
	/// found.
	#[error("Expected s-expr")]
	ExpectedSExpr,

	/// This error will be returned if an opening s-expr with a certain name was expected, but
	/// some other token was found.
	#[error("Expected s-expr identifier {0}")]
	ExpectedSExprIdentifier(&'static str),

	/// This error will be returned if the end of the s-expr was expected, but some other token
	/// was found.
	#[error("Expected end of expression")]
	ExpectedEoe,

	/// This error will be returned if an identifier was expected, but some other token was found.
	#[error("Expected identifier")]
	ExpectedIdentifier,

	/// This error will be returned if a number was expected, but some other token was found.
	#[error("Expected number")]
	ExpectedNumber,

	/// This error will be returned if a string was expected, but some other token was found.
	#[error("Expected string")]
	ExpectedString,

	/// This error will be returned if an s-expr is found, but its name (and fields) were not
	/// supplied to the deserializer (e.g. `deserialize_any` was called).
	#[error("Missing s-expr type info")]
	MissingSExprInfo
}

impl de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Message(msg.to_string())
	}
}
