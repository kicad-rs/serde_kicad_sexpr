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
	let input = r#"(locked)"#;
	let expected = Locked;
	assert_eq_parsed(input, expected);
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename = "uuid")]
struct Uuid(String);

#[test]
fn deserialize_uuid() {
	let input = r#"(uuid "eedfd74b-7d25-4f27-9551-6c4a68de94b9")"#;
	let expected = Uuid("eedfd74b-7d25-4f27-9551-6c4a68de94b9".to_owned());
	assert_eq_parsed(input, expected);
}
