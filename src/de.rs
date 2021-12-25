pub struct Deserializer<'de> {
	input: &'de str
}

impl<'de> Deserializer<'de> {
	pub fn from_str(input: &'de str) -> Self {
		Self { input }
	}
}
