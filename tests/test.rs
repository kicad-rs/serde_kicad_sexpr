use once_cell::sync::Lazy as SyncLazy;
use pretty_assertions::assert_eq;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_sexpr::{deserialize_option, Literal};
use std::fmt::Debug;

fn assert_eq_parsed<T>(input: &str, expected: &T)
where
	T: Debug + DeserializeOwned + PartialEq
{
	let parsed: T = serde_sexpr::from_str(input).expect("Failed to parse input");
	assert_eq!(&parsed, expected);
}

fn assert_eq_written<T>(input: &T, expected: &str)
where
	T: ?Sized + Serialize
{
	let written = serde_sexpr::to_string(input).expect("Failed to write input");
	assert_eq!(written.as_str(), expected);
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "locked")]
struct Locked;

const LOCKED_STR: &str = "(locked)";
static LOCKED_VAL: Locked = Locked;

#[test]
fn deserialize_locked() {
	assert_eq_parsed(LOCKED_STR, &LOCKED_VAL);
}

#[test]
fn serialize_locked() {
	assert_eq_written(&LOCKED_VAL, LOCKED_STR);
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "attr")]
struct Attribute(String);

const ATTRIBUTE_STR: &str = "(attr smd)";
static ATTRIBUTE_VAL: SyncLazy<Attribute> = SyncLazy::new(|| Attribute("smd".to_owned()));

#[test]
fn deserialize_attr() {
	assert_eq_parsed(ATTRIBUTE_STR, &*ATTRIBUTE_VAL);
}

#[test]
fn serialize_attr() {
	assert_eq_written(&*ATTRIBUTE_VAL, ATTRIBUTE_STR);
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "descr")]
struct Description(String);

const DESCRIPTION_STR: &str =
	r#"(descr "Hello \"World\", this \"\\\" is an amazing backspace! \\")"#;
const DESCRIPTION_VAL: SyncLazy<Description> = SyncLazy::new(|| {
	Description(r#"Hello "World", this "\" is an amazing backspace! \"#.to_owned())
});

#[test]
fn deserialize_descr() {
	assert_eq_parsed(DESCRIPTION_STR, &*DESCRIPTION_VAL);
}

#[test]
fn serialize_descr() {
	assert_eq_written(&*DESCRIPTION_VAL, DESCRIPTION_STR);
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "at")]
struct Position {
	x: f32,
	y: f32,
	#[serde(deserialize_with = "deserialize_option")]
	rot: Option<i16>
}

const POSITION_STR_WITHOUT_ROT: &str = "(at 1.23 -4.56)";
static POSITION_VAL_WITHOUT_ROT: Position = Position {
	x: 1.23,
	y: -4.56,
	rot: None
};

const POSITION_STR_WITH_ROT: &str = "(at 1.23 -4.56 -90)";
static POSITION_VAL_WITH_ROT: Position = Position {
	x: 1.23,
	y: -4.56,
	rot: Some(-90)
};

#[test]
fn deserialize_position_without_rot() {
	assert_eq_parsed(POSITION_STR_WITHOUT_ROT, &POSITION_VAL_WITHOUT_ROT);
}

#[test]
fn serialize_position_without_rot() {
	assert_eq_written(&POSITION_VAL_WITHOUT_ROT, POSITION_STR_WITHOUT_ROT);
}

#[test]
fn deserialize_position_with_rot() {
	let input = "(at 1.23 -4.56 -90)";
	let expected = Position {
		x: 1.23,
		y: -4.56,
		rot: Some(-90)
	};
	assert_eq_parsed(input, &expected);
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
	assert_eq_parsed(input, &expected);
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
#[serde(rename = "drill")]
struct Drill {
	oval: bool,
	drill1: f32,
	#[serde(deserialize_with = "deserialize_option")]
	drill2: Option<f32>
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "pad")]
struct Pad {
	index: Literal,
	ty: PadType,
	shape: PadShape,
	at: Position,
	size: Size,
	#[serde(deserialize_with = "deserialize_option")]
	drill: Option<Drill>,
	layers: Vec<String>
}

#[test]
fn deserialize_pad_without_drill() {
	let input = "(pad 1 smd rect (at 0 0) (size 1.27 1.27) (layers F.Cu))";
	let expected = Pad {
		index: 1.into(),
		ty: PadType::Smd,
		shape: PadShape::Rect,
		at: Position {
			x: 0.0,
			y: 0.0,
			rot: None
		},
		size: Size {
			width: 1.27,
			height: 1.27
		},
		drill: None,
		layers: vec!["F.Cu".to_owned()]
	};
	assert_eq_parsed(input, &expected);
}

#[test]
fn deserialize_pad_with_drill() {
	let input = "(pad 1 thru-hole rect (at 0 0) (size 1.27 1.27) (drill 0.635) (layers F.Cu))";
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
		},
		drill: Some(Drill {
			oval: false,
			drill1: 0.635,
			drill2: None
		}),
		layers: vec!["F.Cu".to_owned()]
	};
	assert_eq_parsed(input, &expected);
}

#[test]
fn deserialize_pad_with_oval_drill() {
	let input =
		"(pad 1 thru-hole rect (at 0 0) (size 1.27 1.27) (drill oval 0.635 0.847) (layers F.Cu))";
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
		},
		drill: Some(Drill {
			oval: true,
			drill1: 0.635,
			drill2: Some(0.847)
		}),
		layers: vec!["F.Cu".to_owned()]
	};
	assert_eq_parsed(input, &expected);
}
