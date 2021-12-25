#![warn(rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

mod literal;

pub mod de;

pub use de::from_str;
pub use literal::Literal;
