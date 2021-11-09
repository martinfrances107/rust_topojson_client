#![allow(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![allow(clippy::many_single_char_names)]
//! A port of [topojson/topojson-client](<https://github.com/topojson/topojson-client>).
//!
//! Manipulate TopoJSON, such as to merge shapes, and convert it back to GeoJSON.
//!
//! <hr>
//!
//! Repository [rust_topojson_client](<https://github.com/martinfrances107/rust_topojson_client>)

extern crate derivative;
extern crate topojson;

/// function feature() and various From implementations.
mod feature;

/// function reverse() and unit tests.
mod reverse;

/// function generate, helper type TransformFn and unit tests.
mod transform;
