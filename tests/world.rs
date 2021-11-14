extern crate criterion;
extern crate geo;
extern crate rust_topojson_client;
extern crate topojson;

#[cfg(test)]
mod world_test {

    use std::fs::File;

    use geo::{Geometry, GeometryCollection};
    use rust_topojson_client::feature::Builder;
    use topojson::Topology;

    /// Asserts that a MultiPolygon object with 1428 polygons
    /// can be extracted from the "land" object within the map.
    #[test]
    pub fn object_decode() {
        let file =
            File::open("./tests/world-atlas/world/50m.json").expect("File should open read only.");
        let topology: Topology =
            serde_json::from_reader(file).expect("File should be parse as JSON.");

        match Builder::<f64>::generate_from_name(&topology, &"land") {
            Some(Geometry::GeometryCollection(GeometryCollection(v_geometry))) => {
                assert_eq!(v_geometry.len(), 1);
                match &v_geometry[0] {
                    Geometry::MultiPolygon(mp) => {
                        assert_eq!(mp.0.len(), 1428_usize);
                    }
                    _ => {
                        assert!(false, "Failed to decode Multipoloygon")
                    }
                }
            }
            _ => {
                assert!(false, "failed to extract a vector of geometries");
            }
        };

        // TODO: This fails to parse .. overflows in reverse()
        match Builder::<f64>::generate_from_name(&topology, &"countries") {
            Some(Geometry::GeometryCollection(GeometryCollection(v_geometry))) => {
                assert_eq!(v_geometry.len(), 241);
            }
            _ => {
                assert!(false, "failed to extract a vector of geometries");
            }
        };
    }
}
