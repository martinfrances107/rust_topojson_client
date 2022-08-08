# [Rust D3 topojson client]()

Rust 2021 Edition.

## DESCRIPTION
This is a port of the [topojson](<https://github.com/topojson/topojson>) library into a RUST library crate/package.

The feature extraction section of this library has been ported as is being used actively used in the tests in [rust_d3_geo](https://github.com/martinfrances107/rust_d3_geo)

## CURRENT FOCUS

Developing the binaries and getting topo2geo-test and topoquantize-test to pass.

## TEST STATUS

| Tests          | Status             |
| ---            | ---                |
| feature-test   | tests complete     |
| neigbor-test   | complete           |
| merge-test     | tests incomplete   |
| stich-test          | Module needs work  |
| topo2geo-test     | Missing         |
| topoquantize-test | Missing         |
| transform-test | complete           |
| untransform-test | Module is missing |

## New integration tests

 Additional tests has been added regarding the extraction of an MultiPolygon object named 'land'

 see :-
 test/world.rs
 tests/world-atlas/world/50m.json.

## TODO

* Added criteron benchmarks. based on topo2geo-test
 and topoquantize-test

* Port untranslate test, Maybe?
* neigbours.rs -- is using dynamic dispatch, in an issues related to anomymous functions :-

  ```
  let indexes_by_arc = Rc<RefCell<BTreeMap<usize, ArcIndexes>>>
  ```

  I am leaving this until the benchmarking stage, but I think this can be refactored away.

* Develop some examples, and improve documentation. As an example see  [Africa Lambert Conformal Conic](
  https://bl.ocks.org/bricedev/3905007f1794b0cb0bcd)
