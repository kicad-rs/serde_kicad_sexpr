#![warn(rust_2018_idioms, unreachable_pub)]
#![deny(elided_lifetimes_in_paths)]
#![forbid(unsafe_code)]

//! This crate provides a serde [`Serializer`] and [`Deserializer`] implementation for
//! the S-Expression data format used by KiCAD. Since this format differs in some central
//! aspects from other formats like JSON, there are some limitations and special cases
//! you should be aware of:
//!
//!  - The name of your struct matters. For a simple struct like
//!
//!    ```rust
//!    # use serde::{Deserialize, Serialize};
//!    #[derive(Deserialize, Serialize)]
//!    struct Point(i32, i32);
//!    ```
//!
//!    and an example value `Point(1, 2)` you will get a JSON representation of
//!    `[1, 2]` whereas this crate will output `(Point 1 2)`.
//!
//!  - The name of the fields also matters if the field's type is either a boolean,
//!    a tuple or a sequence. These fields cannot appear in unnamed containers
//!    (i.e. tuple structs).
//!
//!  - Deserializing `Option` is not supported, because we need to know the type inside
//!    the option to determine if it is present or missing. To deserialize optional
//!    values, use the custom deserializing logic from this crate:
//!
//!    ```rust
//!    # use serde::{Deserialize, Serialize};
//!    #[derive(Deserialize, Serialize)]
//!    struct Position {
//!        x: i32,
//!        y: i32,
//!        #[serde(with = "serde_sexpr::Option")]
//!        rotation: Option<i32>
//!    }
//!    ```
//!
//!  - If you need to deserialize some sort of container with an unknown number of
//!    children, use a special field with an empty name, like so:
//!
//!    ```rust
//!    # use serde::{Deserialize, Serialize};
//!    #[derive(Deserialize, Serialize)]
//!    struct Point(i32, i32);
//!
//!    #[derive(Deserialize, Serialize)]
//!    struct Polygon {
//!        #[serde(default, rename = "")]
//!        points: Vec<Point>
//!    }
//!    ```
//!
//!    Note that this has to be the last field of the struct. There must not be any
//!    fields after a field with an empty name, and there must only be one field
//!    with an empty name.
//!
//!  - Untagged enums are not supported. If you need to parse one from a number of
//!    types, use the [`untagged!`] macro:
//!
//!    ```rust
//!    serde_sexpr::untagged! {
//!        enum TextOrNumber {
//!            Text(String),
//!            Int(i32),
//!            Float(f32)
//!        }
//!    }
//!    ```
//!
//!  [`Serializer`]: serde::ser::Serializer
//!  [`Deserializer`]: serde::de::Deserializer
//!  [`untagged!`]: serde_sexpr::untagged

mod option;
#[macro_use]
mod untagged;

pub mod de;
#[doc(hidden)]
pub mod private;
pub mod ser;

pub use de::from_str;
pub use option::{deserialize_option, OptionDef as Option};
pub use ser::{to_string, to_string_pretty};
