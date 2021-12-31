#![warn(rust_2018_idioms, unreachable_pub)]
#![forbid(unsafe_code)]

mod literal;
mod option;
#[macro_use]
mod untagged;

pub mod de;
#[doc(hidden)]
pub mod private;
pub mod ser;

pub use de::from_str;
pub use literal::Literal;
pub use option::{deserialize_option, OptionDef as Option};
pub use ser::{to_string, to_string_pretty};
