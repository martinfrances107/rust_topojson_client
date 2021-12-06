# [Rust D3 topojson client]()

Rust 2021 Edition.

This is a port of the [topojson](<https://github.com/topojson/topojson>) library into a RUST library crate/package.

The only feature extraction section of this library has been ported.

## Ported tests

* transform-test has been completely ported.
* feature-test has been partial ported ..

## New integration tests.

 Additional tests has been added regarding the extraction of an MultiPolygon object named 'land'

 see :-
 test/world.rs
 tests/world-atlas/world/50m.json.



## TODO

Next  port untranslate test, Maybe?
