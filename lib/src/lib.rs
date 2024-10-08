#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]
//! A port of [topojson/topojson-client](<https://github.com/topojson/topojson-client>).
//!
//! Manipulate `TopoJSON` objects, for example merging shapes.
//!
//! <hr>
//!
//! Repository [rust_topojson_client](<https://github.com/martinfrances107/rust_topojson_client>)

extern crate geo;
extern crate geo_types;
#[cfg(test)]
extern crate pretty_assertions;

extern crate topojson;

/// Bounding Box.
mod bbox;
mod bisect;
/// function `feature()` and various From implementations.
pub mod feature;

/// Identifies neighbors in geometry.
pub mod neighbors;

mod feature_geo_type;
mod merge;
mod mesh;
mod polygon_u;
/// function `reverse()` and unit tests.
mod reverse;
mod stitch;
/// function generate, helper type `TransformFn` and unit tests.
mod transform;

/// Translate ARC indexes.
///
/// "A negative arc index indicates that the arc at the ones’ complement of the index must be reversed
/// to reconstruct the geometry: -1 refers to the reversed first arc, -2 refers to the reversed second arc,
/// and so on."
///
/// [source: 2.1.4. Arc Indexes](https://github.com/topojson/topojson-specification#214-arc-indexes)
#[inline]
const fn translate(arc: i32) -> usize {
    if arc < 0 {
        !arc as usize
    } else {
        arc as usize
    }
}

#[cfg(test)]
mod translate_tests {

    use crate::translate;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_double_zero() {
        let input = [-2, -1, 0, 1, 2];
        let mut output = [usize::MIN; 5];
        for (i, arc) in input.iter().enumerate() {
            output[i] = translate(*arc);
        }
        assert_eq!(output, [1, 0, 0, 1, 2]);
    }
}
