use pretty_assertions::assert_eq;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_sexpr::Literal;
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
	let input = "(at 1.23 -4.56)";
	let expected = Position {
		x: 1.23,
		y: -4.56,
		rot: None
	};
	assert_eq_parsed(input, expected);
}

#[test]
fn deserialize_position_with_rot() {
	let input = "(at 1.23 -4.56 -90)";
	let expected = Position {
		x: 1.23,
		y: -4.56,
		rot: Some(-90)
	};
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "size")]
struct Size {
	width: f32,
	height: f32
}

#[test]
fn deserialize_size() {
	let input = "(size 1.23 4.56)";
	let expected = Size {
		width: 1.23,
		height: 4.56
	};
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum PadType {
	#[serde(rename = "thru-hole")]
	ThroughHole,

	#[serde(rename = "smd")]
	Smd
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum PadShape {
	Circle,
	Rect
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "pad")]
struct Pad {
	index: Literal,
	ty: PadType,
	shape: PadShape,
	at: Position,
	size: Size
}

#[test]
fn deserialize_pad() {
	let input = "(pad 1 thru-hole rect (at 0 0) (size 1.27 1.27))";
	let expected = Pad {
		index: 1.into(),
		ty: PadType::ThroughHole,
		shape: PadShape::Rect,
		at: Position {
			x: 0.0,
			y: 0.0,
			rot: None
		},
		size: Size {
			width: 1.27,
			height: 1.27
		}
	};
	assert_eq_parsed(input, expected);
}
