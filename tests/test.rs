use indoc::indoc;
use paste::paste;
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

fn assert_eq_ugly<T>(input: &T, expected: &str)
where
	T: ?Sized + Serialize
{
	let written = serde_sexpr::to_string(input).expect("Failed to write input");
	assert_eq!(written.as_str(), expected);
}

fn assert_eq_pretty<T>(input: &T, expected: &str)
where
	T: ?Sized + Serialize
{
	let written = serde_sexpr::to_string(input).expect("Failed to write input");
	assert_eq!(written.as_str(), expected);
}

macro_rules! test_case {
	(name: $name:ident,input: $input:expr,pretty: $pretty:expr,value: $value:expr) => {
		paste! {
			const [<TEST_CASE_INPUT_ $name:upper>]: &str = $input;
			const [<TEST_CASE_PRETTY_ $name:upper>]: &str = $pretty;

			#[test]
			fn [<test_deserialize_ugly_ $name>]() {
				let value = $value;
				assert_eq_parsed([<TEST_CASE_INPUT_ $name:upper>], &value);
			}

			#[test]
			fn [<test_deserialize_pretty_ $name>]() {
				let value = $value;
				assert_eq_parsed([<TEST_CASE_PRETTY_ $name:upper>], &value);
			}

			#[test]
			fn [<test_serialize_ugly_ $name>]() {
				let value = $value;
				assert_eq_ugly(&value, [<TEST_CASE_INPUT_ $name:upper>]);
			}

			#[test]
			fn [<test_serialize_pretty_ $name>]() {
				let value = $value;
				assert_eq_pretty(&value, [<TEST_CASE_PRETTY_ $name:upper>]);
			}
		}
	};
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "locked")]
struct Locked;

test_case! {
	name: locked,
	input: "(locked)",
	pretty: "(locked)",
	value: Locked
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "attr")]
struct Attribute(String);

test_case! {
	name: attr,
	input: "(attr smd)",
	pretty: "(attr smd)",
	value: Attribute("smd".to_owned())
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "descr")]
struct Description(String);

test_case! {
	name: descr,
	input: r#"(descr "Hello \"World\", this \"\\\" is an amazing backspace! \\")"#,
	pretty: r#"(descr "Hello \"World\", this \"\\\" is an amazing backspace! \\")"#,
	value: Description(r#"Hello "World", this "\" is an amazing backspace! \"#.to_owned())
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

test_case! {
	name: position_without_rot,
	input: "(at 1.23 -4.56)",
	pretty: "(at 1.23 -4.56)",
	value: Position {
		x: 1.23,
		y: -4.56,
		rot: None
	}
}

test_case! {
	name: position_with_rot,
	input: "(at 1.23 -4.56 -90)",
	pretty: "(at 1.23 -4.56 -90)",
	value: Position {
		x: 1.23,
		y: -4.56,
		rot: Some(-90)
	}
}

// ################################################################################################

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "size")]
struct Size {
	width: f32,
	height: f32
}

test_case! {
	name: size,
	input: "(size 1.23 4.56)",
	pretty: "(size 1.23 4.56)",
	value: Size {
		width: 1.23,
		height: 4.56
	}
}

// ################################################################################################

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

test_case! {
	name: pad_without_drill,
	input: "(pad 1 smd rect (at 0 0) (size 1.27 1.27) (layers F.Cu))",
	pretty: indoc!(r#"
		(pad 1 smd rect
		  (at 0 0)
		  (size 1.27 1.27)
		  (layers F.Cu))
	"#),
	value: Pad {
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
	}
}

test_case! {
	name: pad_with_drill,
	input: "(pad 1 thru-hole rect (at 0 0) (size 1.27 1.27) (drill 0.635) (layers F.Cu))",
	pretty: indoc!(r#"
		(pad 1 thru-hole rect
		  (at 0 0)
		  (size 1.27 1.27)
		  (drill 0.635)
		  (layers F.Cu))
	"#),
	value: Pad {
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
	}
}

test_case! {
	name: pad_with_oval_drill,
	input: "(pad 1 thru-hole rect (at 0 0) (size 1.27 1.27) (drill oval 0.635 0.847) (layers F.Cu))",
	pretty: indoc!(r#"
		(pad 1 thru-hole rect
		  (at 0 0)
		  (size 1.27 1.27)
		  (drill oval 0.635 0.847)
		  (layers F.Cu))
	"#),
	value: Pad {
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
	}
}
