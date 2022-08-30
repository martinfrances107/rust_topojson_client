# [Rust D3 topojson client](https://github.com/martinfrances107/rust_topojson_client)

Rust 2021 Edition.

## DESCRIPTION

This is a port of the [topojson](<https://github.com/topojson/topojson>) library into a RUST library crate/package.

The feature extraction section of this library has been ported as is being used actively used in the tests in [rust_d3_geo](https://github.com/martinfrances107/rust_d3_geo)

## CURRENT FOCUS

* Implementing mesh.js and mesh-test.js, which will improve the code covergae in stitch.rs

* Next,  the three binaries

  * topo2geo -106 lines to port
  * topomerge - 216 to port
  * topoquantize - 75 to port
  
  <br>
  Plus associated tests, developing topo2geo-test and topoquantize-test 
## TEST STATUS

  | Tests             | Status            | Comments                        |
  | ----------------- | ----------------- | ------------------------------- |
  | bbox-test         | Complete          |                                 |
  | feature-test      | Complete          |                                 |
  | neigbor-test      | Complete          |                                 |
  | merge-test*       | Complete          |                                 |
  | mesh-test         | Missing code      |                                 |
  | neighbours-test   | Complete          | 53 lines of code to port        |
  | quantize-test     |                   | 50 lines of javascript to port. |
  | topo2geo-test     | Missing           |                                 |
  | topoquantize-test | Missing           |                                 |
  | transform-test    | Complete          |                                 |
  | untransform-test  | Module is missing |                                 |

* merge-tests also act as a test of stitch.rs, although to a limited extent ( code coverage of stitch.rs is 58% ).
implementing mesh-test will increase code coverage.

## New integration tests

 Additional tests has been added regarding the extraction of an MultiPolygon object named 'land'
  see :-
  test/world.rs
  tests/world-atlas/world/50m.json.

## TODO

* Added criteron benchmarks. based on topo2geo-test
 and topoquantize-test

* Port untranslate test, Maybe?
* neigbors.rs -- is using dynamic dispatch, in an issues involving the use of anomymous functions :-

  ```rust
  let indexes_by_arc = RefCell<BTreeMap<usize, ArcIndexes>>
  ```

  I am leaving this until the benchmarking stage, but I think this can be refactored away.

* Develop some examples, and improve documentation. As an example see  [Africa Lambert Conformal Conic](
  https://bl.ocks.org/bricedev/3905007f1794b0cb0bcd)
