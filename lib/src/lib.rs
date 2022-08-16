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
extern crate geo;
#[cfg(test)]
extern crate pretty_assertions;

extern crate topojson;

/// Bounding Box.
mod bbox;
mod bisect;
/// function feature() and various From implementations.
pub mod feature;

/// Identifies neighbors in geometry.
pub mod neighbors;

mod merge;
/// function reverse() and unit tests.
mod reverse;
mod stitch;
/// function generate, helper type TransformFn and unit tests.
mod transform;

// Translate ARC indexes which are signed offsets
// into absolute positions in the array.
#[inline]
fn translate(arc: i32) -> usize {
    if arc < 0 {
        !arc as usize
    } else {
        arc as usize
    }
}
