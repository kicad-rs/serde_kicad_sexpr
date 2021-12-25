use serde::{de, ser};
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

	/// This error will be returned if an opening s-expr was expected, but something else was
	/// found at the input.
	#[error("Expected s-expr")]
	ExpectedSExpr,

	/// This error will be returned if an opening s-expr with a certain name was expected, but
	/// something else was found at the input.
	#[error("Expected s-expr identifier {0}")]
	ExpectedSExprIdentifier(&'static str),

	/// This error will be returned if the end of the s-expr was expected, but some other token
	/// was found.
	#[error("Expected end of expression")]
	ExpectedEoe
}

impl de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Message(msg.to_string())
	}
}

impl ser::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Message(msg.to_string())
	}
}
