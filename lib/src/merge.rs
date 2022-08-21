use std::cell::RefCell;
use std::rc::Rc;

use geo::{CoordFloat, Coordinate, Geometry};
use topojson::{ArcIndexes, Topology, Value};

use crate::feature::Builder as FeatureBuilder;
use crate::stitch::stitch;
use crate::translate;

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

/// A polygon which can be marked with a underscore
/// to imply it has been processed.
///
/// In javascript object are dynamic in rust we need
/// this wrapper( or an enum )
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

    #[inline]
    fn is_not_makred(&self) -> bool {
        !self.underscore
    }
}

/// todo
#[derive(Clone, Debug)]
pub struct MergeArcs {
    polygons: Vec<Rc<RefCell<PolygonU>>>,

    // Rc<RefCell<_>>
    // A Shared refeerence is needed here becuase changes to the contents
    // of the 'polygon' refcell should be observed in multiple rows of the
    // polygons_by_arc table.
    polygons_by_arc: Vec<Vec<Rc<RefCell<PolygonU>>>>,

    groups: Vec<Vec<PolygonU>>,
    topology: Topology,
}

impl MergeArcs {
    pub fn new(topology: Topology) -> Self {
        Self {
            polygons_by_arc: vec![],
            polygons: vec![],
            groups: vec![],
            topology,
        }
    }

    // Proces collections of items - 'extract'ing all sub items.
    fn geometry(&mut self, o: &Value) {
        match o {
            Value::GeometryCollection(gc) => {
                gc.iter().for_each(|o| self.geometry(&o.value));
            }
            Value::Polygon(p) => {
                self.extract(p);
            }
            Value::MultiPolygon(mp) => {
                mp.iter().for_each(|x| self.extract(x));
            }
            // Ignore ValuesMultiLines, Values::Lines,  Values::Points etc.
            _ => {}
        }
    }

    /// Loop over the input pushing to internal state.
    /// polygons_by_arc and polygons.    
    fn extract(&mut self, polygon: &[Vec<i32>]) {
        // Value to be stored and refered to .. in pba
        let pu = Rc::new(RefCell::new(PolygonU::new(polygon.to_vec())));

        polygon.iter().for_each(|ring| {
            ring.iter().for_each(|arc| {
                let index = translate(*arc);
                match self.polygons_by_arc.get(index) {
                    Some(_) => self.polygons_by_arc[index].push(pu.clone()),
                    None => {
                        self.polygons_by_arc.insert(index, vec![pu.clone()]);
                    }
                };
            });
        });

        self.polygons.push(pu);
    }

    /// Generate a Polygons using MergeArcs.
    pub fn generate(&mut self, objects: &mut [Value]) -> Value {
        objects.iter().for_each(|o| self.geometry(o));

        self.polygons.clone().iter().for_each(|polygon| {
            if polygon.borrow().is_not_makred() {
                let mut group: Vec<PolygonU> = vec![];

                polygon.borrow_mut().underscore = true;

                let mut neighbors = vec![polygon];

                // Iterate over neighbors while conditionally pushing to the tail.
                while let Some(polygon) = neighbors.pop() {
                    group.push(polygon.borrow().clone());
                    polygon.borrow().v.iter().for_each(|ring| {
                        ring.iter().for_each(|arc| {
                            let index = translate(*arc);
                            self.polygons_by_arc[index].iter().for_each(|polygon| {
                                if polygon.borrow().is_not_makred() {
                                    polygon.borrow_mut().underscore = true;
                                    neighbors.push(polygon);
                                }
                            });
                        });
                    });
                }
                self.groups.push(group);
            }
        });

        self.polygons
            .iter_mut()
            .for_each(|polygon| polygon.borrow_mut().underscore = false);

        // Extract the exterior (unique) arcs.
        let polygon_arcs = self
            .groups
            .iter()
            .map(|polygons| {
                // todo can I use with_capacity() here.
                let mut arcs = Vec::new();
                polygons.iter().for_each(|polygon| {
                    polygon.v.iter().for_each(|ring| {
                        ring.iter().for_each(|arc| {
                            let index = translate(*arc);
                            if self.polygons_by_arc[index].len() < 2 {
                                arcs.push(*arc);
                            }
                        });
                    });
                });

                // Stich the arc into one or more rings.
                let mut arcs = stitch(&self.topology, arcs);

                // If more than one ring is returned,
                // at most one of these rings can be the exterior;
                // choose the one with the greatest absolute area.
                if !arcs.is_empty() {
                    let mut iter_mut = arcs.iter_mut();
                    let mut k = self.area(iter_mut.next().unwrap().to_vec());
                    let mut ki;
                    let mut t;
                    for a in arcs.clone().iter_mut() {
                        ki = self.area(a.to_vec());
                        if ki > k {
                            // todo this is a swap with arcs[0]
                            t = a.clone();
                            arcs[0] = a.clone();
                            *a = t;
                            k = ki;
                        }
                    }
                }
                arcs
            })
            .filter(|arcs| !(*arcs).is_empty())
            .collect();

        Value::MultiPolygon(polygon_arcs)
    }

    fn area(&self, ring: ArcIndexes) -> f64 {
        let polygon = Value::Polygon(vec![ring]);
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
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::Topology;
    use topojson::TransformParams;
    use topojson::Value;

    use super::MergeArcs;

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
    //     println!("merge stitches together two side-by-side polygons");
    //     let mut values = vec![
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
    //         transform: None,
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

    //     assert_eq!(MergeArcs::new(topology).generate(&mut values), mp);
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
