use geo::{CoordFloat, Coordinate, Geometry};
use topojson::{ArcIndexes, Topology, Value};

use super::stitch::Stitch;
use crate::feature::Builder as FeatureBuilder;

fn planar_ring_area<T>(ring: &Vec<Coordinate<T>>) -> T
where
    T: CoordFloat,
{
    let mut a: Coordinate<T>;
    let mut b: Coordinate<T> = *ring.last().unwrap();
    let mut area = T::zero();
    for r in ring {
        a = b;
        b = *r;
        area = area + a.x * b.y - a.y * b.x;
    }
    area.abs() // Note: doubled area!
}

#[derive(Clone, Debug)]
struct PolygonU {
    v: Vec<ArcIndexes>,
    underscore: bool,
}

impl PolygonU {
    fn new(v: Vec<ArcIndexes>) -> Self {
        Self {
            v,
            underscore: false,
        }
    }
}

/// todo
#[derive(Clone, Debug)]
pub struct MergeArcs {
    polygons_by_arc: Vec<Vec<PolygonU>>,
    polygons: Vec<PolygonU>,
    groups: Vec<Vec<Vec<ArcIndexes>>>,
    topology: Topology,
}

impl MergeArcs {
    fn new(topology: Topology) -> Self {
        Self {
            polygons_by_arc: vec![],
            polygons: vec![],
            groups: vec![],
            topology,
        }
    }

    fn geometry(&mut self, o: &mut Value) {
        match o {
            Value::GeometryCollection(gc) => {
                gc.iter_mut().for_each(|o| self.geometry(&mut o.value));
            }
            Value::Polygon(p) => {
                self.extract(p);
            }
            Value::MultiPolygon(mp) => {
                mp.iter_mut().for_each(|x| self.extract(x));
            }
            _ => {}
        }
    }

    /// Loop over the input pushing to internal state.
    /// polygons_by_arc and polygons.
    /// In the original JS objects are dynamic
    /// So I have modified to convert polygon into polygon_u
    /// which potential double the memory requirements.
    /// Thinking about using drain.
    fn extract(&mut self, polygon: &[Vec<i32>]) {
        polygon.iter().for_each(|ring| {
            ring.iter().for_each(|arc| {
                let index = if *arc < 0 {
                    !*arc as usize
                } else {
                    *arc as usize
                };
                match self.polygons_by_arc.get(index) {
                    Some(_) => self.polygons_by_arc[index].push(PolygonU::new(polygon.to_vec())),
                    None => {
                        self.polygons_by_arc[index] = vec![];
                        self.polygons_by_arc[index].push(PolygonU::new(polygon.to_vec()))
                    }
                };
            });
        });
        self.polygons.push(PolygonU::new(polygon.to_vec()));
    }

    fn appply_geometry(mut self, objects: &mut [Value]) -> Self {
        objects.iter_mut().for_each(|o| self.geometry(o));
        self
    }

    /// Generate a Polygons using MergeArcs.
    fn generate(topology: Topology, objects: &[Value]) -> Value {
        let mut ma = Self::new(topology);

        ma = ma.appply_geometry(&mut objects.to_owned());

        ma.polygons.iter_mut().for_each(|polygon| {
            if !polygon.underscore {
                let mut group = vec![];
                let mut neighbors = vec![polygon.clone()];

                polygon.underscore = true;
                // ma.groups.push(group.clone());
                while neighbors.pop().is_some() {
                    group.push(polygon.v.clone());
                    polygon.v.iter().for_each(|ring| {
                        ring.iter().for_each(|arc| {
                            let index = if *arc < 0 {
                                !*arc as usize
                            } else {
                                *arc as usize
                            };
                            // ma.polygons_by_arc[index].iter_mut().for_each(|polygon| {
                            //     if !polygon.underscore {
                            //         polygon.underscore = true;
                            //         neighbors.push(polygon.clone());
                            //     }
                            // });
                        });
                    });
                }
            }
        });

        ma.polygons
            .iter_mut()
            .for_each(|polygon| polygon.underscore = false);

        let arcs: Vec<ArcIndexes> = vec![];
        // let arcs = ma
        //     .groups
        //     .iter()
        //     .map(|polygons| {
        //         let arcs = Vec::new();
        //         let n: usize;
        //         ma.polygons.iter().map(|polygon| {
        //             polygon.v.iter().map(|ring| {
        //                 ring.iter().map(|arc| {
        //                     let arc = if *arc < 0_i32 { !*arc } else { *arc };
        //                     if ma.polygons_by_arc[arc as usize].len() < 2 {}
        //                 })
        //             });
        //         });

        //         // Stich the arc into one or more rings.
        //         arcs = Stitch::default().gen(ma.topology, arcs);

        //         // If more than one ring is returned,
        //         // at most one of these rings can be the exterior;
        //         // choose the one with the greatest absolute area.
        //         n = arcs.len();
        //         if n > 1 {
        //             let t;
        //             let iter = arcs.iter_mut();
        //             let k = ma.area(*iter.next().unwrap());
        //             let ki;
        //             let t;
        //             for a in iter {
        //                 ki = ma.area(a);
        //                 if ki > k {
        //                     // todo this is a swap with arcs[0]
        //                     t = a;
        //                     arcs[0] = *a;
        //                     a = t;
        //                     k = ki;
        //                 }
        //             }
        //         }
        //         Value::MultiPolygon(arcs)
        //     })
        //     .filter(|arcs| (*arcs).len() > 0)
        //     .collect();

        Value::Polygon(arcs)
    }

    fn area(&self, ring: Vec<ArcIndexes>) -> f64 {
        let polygon = Value::Polygon(ring);
        let object = FeatureBuilder::generate(&self.topology, &polygon);
        match object {
            Geometry::Polygon(p) => planar_ring_area(&p.exterior().0),
            _ => {
                todo!("was expecting a polygon");
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod merge_tests {

    use crate::merge;

    use super::*;
    use geo::line_string;
    use geo::Geometry;
    use geo::MultiPolygon;
    use geo::Polygon;
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::TransformParams;
    use topojson::Value;

    // tape("merge ignores null geometries", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {},
    //       "arcs": []
    //     };
    //     test.deepEqual(topojson.merge(topology, [{type: null}]), {
    //       type: "MultiPolygon",
    //       coordinates: []
    //     });
    //     test.end();
    //   });

    //
    // +----+----+            +----+----+
    // |    |    |            |         |
    // |    |    |    ==>     |         |
    // |    |    |            |         |
    // +----+----+            +----+----+
    //
    //   tape("merge stitches together two side-by-side polygons", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0, 1]]},
    //             {"type": "Polygon", "arcs": [[-1, 2]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[1, 1], [1, 0]],
    //         [[1, 0], [0, 0], [0, 1], [1, 1]],
    //         [[1, 1], [2, 1], [2, 0], [1, 0]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[1, 0], [0, 0], [0, 1], [1, 1], [2, 1], [2, 0], [1, 0]]]]
    //     });
    //     test.end();
    //   });

    //
    // +----+----+            +----+----+
    // |    |    |            |         |
    // |    |    |    ==>     |         |
    // |    |    |            |         |
    // +----+----+            +----+----+
    //
    // #[test]
    // fn stitches_together_two_side_by_side_polygons() {
    //     let values = vec![
    //         Value::Polygon(vec![vec![0, 1]]),
    //         Value::Polygon(vec![vec![-1, 2]]),
    //     ];
    //     let polys = vec![
    //         topojson::Geometry::new(Value::Polygon(vec![vec![0, 1]])),
    //         topojson::Geometry::new(Value::Polygon(vec![vec![-1, 2]])),
    //     ];
    //     let object = Value::GeometryCollection(polys);
    //     let topology = Topology {
    //         arcs: vec![
    //             vec![vec![1_f64, 1_f64], vec![1_f64, 0_f64]],
    //             vec![
    //                 vec![1_f64, 0_f64],
    //                 vec![0_f64, 0_f64],
    //                 vec![0_f64, 1_f64],
    //                 vec![1_f64, 1_f64],
    //             ],
    //             vec![
    //                 vec![1_f64, 1_f64],
    //                 vec![2_f64, 1_f64],
    //                 vec![2_f64, 0_f64],
    //                 vec![1_f64, 0_f64],
    //             ],
    //         ],
    //         objects: vec![NamedGeometry {
    //             name: "foo".to_string(),
    //             geometry: topojson::Geometry::new(object),
    //         }],
    //         bbox: None,
    //         transform: Some(TransformParams {
    //             scale: [1_f64, 1_f64],
    //             translate: [0_f64, 0_f64],
    //         }),
    //         foreign_members: None,
    //     };
    //     let mp = Value::MultiPolygon(vec![vec![
    //         vec![1, 0],
    //         vec![0, 0],
    //         vec![0, 1],
    //         vec![1, 1],
    //         vec![2, 1],
    //         vec![2, 0],
    //         vec![1, 0],
    //     ]]);

    //     assert_eq!(MergeArcs::generate(topology, &values), mp);
    // }

    //   //
    //   // +----+----+            +----+----+
    //   // |    |    |            |         |
    //   // |    |    |    ==>     |         |
    //   // |    |    |            |         |
    //   // +----+----+            +----+----+
    //   //
    //   tape("merge stitches together geometry collections", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0, 1]]},
    //             {"type": "Polygon", "arcs": [[-1, 2]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[1, 1], [1, 0]],
    //         [[1, 0], [0, 0], [0, 1], [1, 1]],
    //         [[1, 1], [2, 1], [2, 0], [1, 0]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, [topology.objects.collection]), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[1, 0], [0, 0], [0, 1], [1, 1], [2, 1], [2, 0], [1, 0]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +----+ +----+            +----+ +----+
    //   // |    | |    |            |    | |    |
    //   // |    | |    |    ==>     |    | |    |
    //   // |    | |    |            |    | |    |
    //   // +----+ +----+            +----+ +----+
    //   //
    //   tape("merge does not stitch together two separated polygons", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0]]},
    //             {"type": "Polygon", "arcs": [[1]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[0, 0], [0, 1], [1, 1], [1, 0], [0, 0]],
    //         [[2, 0], [2, 1], [3, 1], [3, 0], [2, 0]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[0, 0], [0, 1], [1, 1], [1, 0], [0, 0]]], [[[2, 0], [2, 1], [3, 1], [3, 0], [2, 0]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +-----------+            +-----------+
    //   // |           |            |           |
    //   // |   +---+   |    ==>     |   +---+   |
    //   // |   |   |   |            |   |   |   |
    //   // |   +---+   |            |   +---+   |
    //   // |           |            |           |
    //   // +-----------+            +-----------+
    //   //
    //   tape("merge does not stitch together a polygon and its hole", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0], [1]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],
    //         [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]], [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +-----------+            +-----------+
    //   // |           |            |           |
    //   // |   +---+   |    ==>     |           |
    //   // |   |   |   |            |           |
    //   // |   +---+   |            |           |
    //   // |           |            |           |
    //   // +-----------+            +-----------+
    //   //
    //   tape("merge stitches together a polygon surrounding another polygon", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0], [1]]},
    //             {"type": "Polygon", "arcs": [[-2]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],
    //         [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +-----------+-----------+            +-----------+-----------+
    //   // |           |           |            |                       |
    //   // |   +---+   |   +---+   |    ==>     |   +---+       +---+   |
    //   // |   |   |   |   |   |   |            |   |   |       |   |   |
    //   // |   +---+   |   +---+   |            |   +---+       +---+   |
    //   // |           |           |            |                       |
    //   // +-----------+-----------+            +-----------+-----------+
    //   //
    //   tape("merge stitches together two side-by-side polygons with holes", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0, 1], [2]]},
    //             {"type": "Polygon", "arcs": [[-1, 3], [4]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[3, 3], [3, 0]],
    //         [[3, 0], [0, 0], [0, 3], [3, 3]],
    //         [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]],
    //         [[3, 3], [6, 3], [6, 0], [3, 0]],
    //         [[4, 1], [5, 1], [5, 2], [4, 2], [4, 1]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[3, 0], [0, 0], [0, 3], [3, 3], [6, 3], [6, 0], [3, 0]], [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]], [[4, 1], [5, 1], [5, 2], [4, 2], [4, 1]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +-------+-------+            +-------+-------+
    //   // |       |       |            |               |
    //   // |   +---+---+   |    ==>     |   +---+---+   |
    //   // |   |       |   |            |   |       |   |
    //   // |   +---+---+   |            |   +---+---+   |
    //   // |       |       |            |               |
    //   // +-------+-------+            +-------+-------+
    //   //
    //   tape("merge stitches together two horseshoe polygons, creating a hole", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0, 1, 2, 3]]},
    //             {"type": "Polygon", "arcs": [[-3, 4, -1, 5]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[2, 3], [2, 2]],
    //         [[2, 2], [1, 2], [1, 1], [2, 1]],
    //         [[2, 1], [2, 0]],
    //         [[2, 0], [0, 0], [0, 3], [2, 3]],
    //         [[2, 1], [3, 1], [3, 2], [2, 2]],
    //         [[2, 3], [4, 3], [4, 0], [2, 0]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[2, 0], [0, 0], [0, 3], [2, 3], [4, 3], [4, 0], [2, 0]], [[2, 2], [1, 2], [1, 1], [2, 1], [3, 1], [3, 2], [2, 2]]]]
    //     });
    //     test.end();
    //   });

    //   //
    //   // +-------+-------+            +-------+-------+
    //   // |       |       |            |               |
    //   // |   +---+---+   |    ==>     |               |
    //   // |   |   |   |   |            |               |
    //   // |   +---+---+   |            |               |
    //   // |       |       |            |               |
    //   // +-------+-------+            +-------+-------+
    //   //
    //   tape("merge stitches together two horseshoe polygons surrounding two other polygons", function(test) {
    //     var topology = {
    //       "type": "Topology",
    //       "objects": {
    //         "collection": {
    //           "type": "GeometryCollection",
    //           "geometries": [
    //             {"type": "Polygon", "arcs": [[0, 1, 2, 3]]},
    //             {"type": "Polygon", "arcs": [[-3, 4, -1, 5]]},
    //             {"type": "Polygon", "arcs": [[6, -2]]},
    //             {"type": "Polygon", "arcs": [[-7, -5]]}
    //           ]
    //         }
    //       },
    //       "arcs": [
    //         [[2, 3], [2, 2]],
    //         [[2, 2], [1, 2], [1, 1], [2, 1]],
    //         [[2, 1], [2, 0]],
    //         [[2, 0], [0, 0], [0, 3], [2, 3]],
    //         [[2, 1], [3, 1], [3, 2], [2, 2]],
    //         [[2, 3], [4, 3], [4, 0], [2, 0]],
    //         [[2, 2], [2, 1]]
    //       ]
    //     };
    //     test.deepEqual(topojson.merge(topology, topology.objects.collection.geometries), {
    //       type: "MultiPolygon",
    //       coordinates: [[[[2, 0], [0, 0], [0, 3], [2, 3], [4, 3], [4, 0], [2, 0]]]]
    //     });
    //     test.end();
    //   });
}
