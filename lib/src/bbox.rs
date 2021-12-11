use crate::transform::Transform;
use topojson::{NamedGeometry, Topology, Value};

#[derive(Clone, Debug)]
pub struct BBox {
    t: Transform,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl BBox {
    #[inline]
    pub fn calc(topology: &Topology) -> [f64; 4] {
        match &topology.transform {
            // Without a transform present ..return the widest bounding box.
            None => [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            Some(transform) => {
                let mut state = BBox {
                    t: Transform::new(transform),
                    x0: f64::INFINITY,
                    y0: f64::INFINITY,
                    x1: f64::NEG_INFINITY,
                    y1: f64::NEG_INFINITY,
                };

                for arc in &topology.arcs {
                    for (i, a) in arc.iter().enumerate() {
                        let p = state.t.transform(a, i);
                        if p[0] < state.x0 {
                            state.x0 = p[0];
                        }
                        if p[0] > state.x1 {
                            state.x1 = p[0];
                        }
                        if p[1] < state.y0 {
                            state.y0 = p[1];
                        }
                        if p[1] > state.y1 {
                            state.y1 = p[1];
                        }
                    }
                }

                for key in &topology.objects {
                    state.bbox_geometry(key)
                }

                [state.x0, state.y0, state.x1, state.y1]
            }
        }
    }

    fn bbox_point(&mut self, p: &[f64]) {
        let p = self.t.transform(p, 0);
        if p[0] < self.x0 {
            self.x0 = p[0];
        }
        if p[0] > self.x1 {
            self.x1 = p[0];
        }
        if p[1] < self.y0 {
            self.y0 = p[1];
        }
        if p[1] > self.y1 {
            self.y1 = p[1];
        }
    }

    fn bbox_geometry(&mut self, o: &NamedGeometry) {
        match &o.geometry.value {
            Value::GeometryCollection(vg) => {
                for g in vg {
                    self.bbox_geometry(&NamedGeometry {
                        name: "i".to_string(),
                        geometry: g.clone(),
                    });
                }
            }
            Value::Point(p) => {
                self.bbox_point(p);
            }
            Value::MultiPoint(mp) => {}
            _ => {
                unimplemented!("Can I skip this");
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod bbox_tests {
    use std::fs::File;
    use std::io::Read;

    extern crate serde;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn ignores_the_exiting_bbox() {
        println!("topojson.bbox(topology) ignores the existing bbox, if any");

        assert_eq!(
            BBox::calc(&Topology {
                arcs: vec![],
                objects: vec![],
                bbox: Some(vec![1_f64, 2_f64, 3_f64, 4_f64]),
                transform: None,
                foreign_members: None,
            }),
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY
            ]
        );
    }

    #[test]
    fn computes_for_quantized_topology() {
        println!("topojson.bbox(topology) computes the bbox for a quantized topology, if missing");
        let mut file =
            File::open("./tests/topojson/polygon-q1e4.json").expect("could not load json file.");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("did not read file correctly.");

        let topology: Topology = serde_json::from_str(&data).expect("did not parse correcly");
        assert_eq!(BBox::calc(&topology), [0_f64, 0_f64, 10_f64, 10_f64]);
    }

    #[test]
    fn computes_the_bbox_for_a_non_quantized_topology_if_missing() {
        println!(
            "topojson.bbox(topology) computes the bbox for a non-quantized topology, if missing"
        );
        let mut file =
            File::open("./tests/topojson/polygon.json").expect("could not load json file.");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("did not read file correctly.");

        let topology: Topology = serde_json::from_str(&data).expect("did not parse correcly");
        assert_eq!(BBox::calc(&topology), [0_f64, 0_f64, 10_f64, 10_f64]);
    }

    #[test]
    fn computes_the_bbox_considers_points() {
        println!("topojson.bbox(topology) considers points");
        let mut file =
            File::open("./tests/topojson/point.json").expect("could not load json file.");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("did not read file correctly.");

        let topology: Topology = serde_json::from_str(&data).expect("did not parse correcly");
        assert_eq!(BBox::calc(&topology), [0_f64, 0_f64, 10_f64, 10_f64]);
    }

    #[test]
    fn considers_multipoints() {
        println!("topojson.bbox(topology) considers multipoints");
        let mut file =
            File::open("./tests/topojson/points.json").expect("could not load json file.");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("did not read file correctly.");

        let topology: Topology = serde_json::from_str(&data).expect("did not parse correcly");
        assert_eq!(BBox::calc(&topology), [0_f64, 0_f64, 10_f64, 10_f64]);
    }
}
