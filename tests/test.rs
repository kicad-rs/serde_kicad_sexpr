use pretty_assertions::assert_eq;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

fn assert_eq_parsed<T>(input: &str, expected: T)
where
	T: Debug + DeserializeOwned + PartialEq
{
	let parsed: T = serde_sexpr::from_str(input).expect("Failed to parse input");
	assert_eq!(parsed, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "locked")]
struct Locked;

#[test]
fn deserialize_locked() {
	let input = "(locked)";
	let expected = Locked;
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "attr")]
struct Attribute(String);

#[test]
fn deserialize_attr() {
	let input = "(attr smd)";
	let expected = Attribute("smd".to_owned());
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "descr")]
struct Description(String);

#[test]
fn deserialize_descr() {
	let input = r#"(descr "Hello \"World\", this \"\\\" is an amazing backspace! \\")"#;
	let expected = Description(r#"Hello "World", this "\" is an amazing backspace! \"#.to_owned());
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "at")]
struct Position {
	x: f32,
	y: f32,
	rot: Option<i16>
}

#[test]
fn deserialize_position_without_rot() {
	let input = "(at 1.23 4.56)";
	let expected = Position {
		x: 1.23,
		y: 4.56,
		rot: None
	};
	assert_eq_parsed(input, expected);
}

#[test]
fn deserialize_position_with_rot() {
	let input = "(at 1.23 4.56 -90)";
	let expected = Position {
		x: 1.23,
		y: 4.56,
		rot: Some(-90)
	};
	assert_eq_parsed(input, expected);
}
