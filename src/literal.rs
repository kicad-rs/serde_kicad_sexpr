use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Eq, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
enum LiteralImp {
	Number(u16),
	Text(String)
}

#[derive(Clone, Eq, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Literal(LiteralImp);

impl Debug for Literal {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self.0 {
			LiteralImp::Number(num) => Debug::fmt(num, f),
			LiteralImp::Text(text) => Debug::fmt(text, f)
		}
	}
}

impl Display for Literal {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self.0 {
			LiteralImp::Number(num) => Display::fmt(num, f),
			LiteralImp::Text(text) => Display::fmt(text, f)
		}
	}
}

impl From<u16> for Literal {
	fn from(num: u16) -> Self {
		Self(LiteralImp::Number(num))
	}
}

impl From<&str> for Literal {
	fn from(text: &str) -> Self {
		text.to_owned().into()
	}
}

impl From<String> for Literal {
	fn from(text: String) -> Self {
		Self(LiteralImp::Text(text))
	}
}
