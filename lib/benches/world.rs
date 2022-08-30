extern crate criterion;
extern crate geo;
extern crate rust_topojson_client;
extern crate topojson;

use std::fs::File;
use std::io::Read;

use criterion::{criterion_group, criterion_main, Criterion};
use geo::{Geometry, GeometryCollection};
use topojson::Topology;

use rust_topojson_client::feature::feature_from_name;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut file = File::open("./tests/world-atlas/world/50m.json").expect("File did not open.");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file.");

    let topology: Topology = serde_json::from_str(&contents).expect("Failed to read as json.");

    c.bench_function("world", |b| {
        b.iter(|| {
            match feature_from_name::<f64>(&topology, &"land") {
                Some(Geometry::GeometryCollection(GeometryCollection(v_geometry))) => {
                    assert_eq!(v_geometry.len(), 1);
                    match &v_geometry[0] {
                        Geometry::MultiPolygon(mp) => {
                            assert_eq!(mp.0.len(), 1428_usize);
                        }
                        _ => {
                            assert!(false, "Failed to decode Multi poloygon")
                        }
                    }
                }
                _ => {
                    assert!(false, "failed to extract a vector of geometries");
                }
            };
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
