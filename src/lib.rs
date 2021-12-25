#![warn(rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

pub mod de;

pub use de::from_str;
