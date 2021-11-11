extern crate criterion;
extern crate geo;
extern crate rust_topojson_client;
extern crate topojson;

#[cfg(test)]
mod world_test {
    use std::env;
    use std::fs::File;
    use std::io::Read;

    use geo::{Geometry, GeometryCollection};
    use rust_topojson_client::feature::Builder;
    use topojson::Topology;

    /// Asserts that a MultiPolygon object with 1428 polygons
    /// can be extracted from the "land" object within the map.
    #[test]
    pub fn land_decode() {
        let mut file =
            File::open("./tests/world-atlas/world/50m.json").expect("File did not open.");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read file.");

        let topology: Topology = serde_json::from_str(&contents).expect("Faile to read as json.");

        dbg!(&topology.transform);

        let computed = Builder::<f64>::generate_from_name(&topology, &"land");

        match computed {
            Some(Geometry::GeometryCollection(GeometryCollection(v_geometry))) => {
                assert_eq!(v_geometry.len(), 1);
                match &v_geometry[0] {
                    Geometry::MultiPolygon(mp) => {
                        assert_eq!(mp.0.len(), 1428_usize);
                    }
                    _ => {
                        assert!(false, "Faile to decode Multipoloygon")
                    }
                }
            }
            _ => {
                assert!(false, "failed to extract a vector of geometries");
            }
        };
    }
}
