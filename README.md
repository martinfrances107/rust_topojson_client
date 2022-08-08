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

* get command lines working, by using [clap](https://docs.rs/clap/3.2.16/clap/).
* neigbours.rs -- is using dynamic dispatch, in an issues related to anomymous functions :-

```
let indexes_by_arc = Rc<RefCell<BTreeMap<usize, ArcIndexes>>>
```

I am leaving this until the benchmarking stage, but I think this can be refactored away.



* Next  port untranslate test, Maybe?


* develop a example, using a small map of africa

  https://bl.ocks.org/bricedev/3905007f1794b0cb0bcd
