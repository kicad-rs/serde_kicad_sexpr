use serde::ser;
use std::fmt::Display;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
	#[error("{0}")]
	Message(String),

	/// This error will be returned if you request to serialize anything but a struct at root
	/// level.
	#[error("Expected to serialize a struct at root level")]
	ExpectedStruct,

	/// This error will be returned if a boolean was detected in an unnamed container, i.e.
	/// a tuple or a sequence.
	#[error("Unnamed boolean")]
	UnnamedBoolean,

	/// This error will be returned if a sequence was detected in an unnamed container, i.e. a
	/// tuple or a sequence.
	#[error("Unnamed sequence")]
	UnnamedSeq,

	#[error("char is unsupported")]
	Char,
	#[error("byte array is unsupported")]
	Bytes,
	#[error("unit is unsupported")]
	Unit,
	#[error("enums with non-unit variants are not supported")]
	ComplexEnum,
	#[error("maps are not supported")]
	Map
}

impl ser::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Message(msg.to_string())
	}
}
