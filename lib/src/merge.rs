use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::rc::Rc;

use geo::{Coord, CoordFloat, Geometry};
use topojson::{ArcIndexes, NamedGeometry, Topology, Value};

use crate::feature::feature;
use crate::polygon_u::PolygonU;
use crate::stitch::stitch;
use crate::translate;

fn planar_ring_area<T>(ring: &Vec<Coord<T>>) -> T
where
    T: CoordFloat,
{
    let mut a: Coord<T>;
    let mut b: Coord<T> = *ring.last().unwrap();
    let mut area = T::zero();
    for r in ring {
        a = b;
        b = *r;
        area = area + a.x * b.y - a.y * b.x;
    }
    area.abs() // Note: doubled area!
}

/// Given a topology and list of objects, merge the selected objected together, translate and output
/// a resulting object as `geo_types::Geometry` object.
pub fn merge<T>(topology: &Topology, objects: &[NamedGeometry]) -> Geometry<T>
where
    T: CoordFloat + Debug,
{
    let mut ma = MergeArcs::new(topology);

    objects.iter().for_each(|o| ma.geometry(&o.geometry));

    ma.polygons.clone().iter().for_each(|polygon| {
        if polygon.borrow().is_not_marked() {
            let mut group: Vec<PolygonU> = vec![];

            polygon.borrow_mut().mark();

            let mut neighbors = vec![polygon];

            // Iterate over neighbors while conditionally pushing to the tail.
            while let Some(polygon) = neighbors.pop() {
                group.push(polygon.borrow().clone());
                polygon.borrow().v.iter().for_each(|ring| {
                    for arc in ring {
                        let index = translate(*arc);
                        ma.polygons_by_arc[&index].iter().for_each(|polygon| {
                            if polygon.borrow().is_not_marked() {
                                polygon.borrow_mut().mark();
                                neighbors.push(polygon);
                            }
                        });
                    }
                });
            }
            ma.groups.push(group);
        }
    });

    ma.polygons
        .iter_mut()
        .for_each(|polygon| polygon.borrow_mut().unmark());

    // Extract the exterior (unique) arcs.
    let polygon_arcs = ma
        .groups
        .iter()
        .map(|polygons| {
            // todo can I use with_capacity() here.
            let mut arcs = Vec::new();
            for polygon in polygons {
                polygon.v.iter().for_each(|ring| {
                    for arc in ring {
                        let index = translate(*arc);
                        if ma.polygons_by_arc[&index].len() < 2 {
                            arcs.push(*arc);
                        }
                    }
                });
            }

            // Stich the arc into one or more rings.
            let mut arcs = stitch(topology, arcs);
            // If more than one ring is returned, at most one of these
            // rings can be the exterior; choose the one with the
            // greatest absolute area.
            let n = arcs.len();
            if n > 1 {
                let mut iter_mut = arcs.iter_mut();
                let mut k = ma.area(iter_mut.next().unwrap().clone());
                let mut ki;
                for i in 1..arcs.len() {
                    ki = ma.area(arcs[i].clone());
                    if ki > k {
                        arcs.swap(0, i);
                        k = ki;
                    }
                }
            }
            arcs
        })
        .filter(|arcs| !(*arcs).is_empty())
        .collect();
    let ma = Value::MultiPolygon(polygon_arcs);

    // let merge_arcs = ma.generate(objects);
    feature(topology, &ma)
}

#[derive(Debug)]
struct MergeArcs<'a> {
    polygons: Vec<Rc<RefCell<PolygonU>>>,

    // Rc<RefCell<_>> A Shared refeerence is needed here becuase changes to
    // the contents of the 'polygon' refcell should be observed in multiple
    // rows of the polygons_by_arc table.
    polygons_by_arc: BTreeMap<usize, Vec<Rc<RefCell<PolygonU>>>>,

    groups: Vec<Vec<PolygonU>>,
    topology: &'a Topology,
}

impl<'a> MergeArcs<'a> {
    fn new(topology: &'a Topology) -> Self {
        Self {
            polygons: vec![],
            polygons_by_arc: BTreeMap::new(),
            groups: vec![],
            topology,
        }
    }

    // Proces collections of items - 'extract'ing all sub items.
    fn geometry(&mut self, o: &topojson::Geometry) {
        // let value = o.value;
        match &o.value {
            Value::GeometryCollection(gc) => {
                for g in gc {
                    self.geometry(g);
                }
            }
            Value::Polygon(polygon) => self.extract(polygon),
            Value::MultiPolygon(mp) => {
                for p in mp {
                    self.extract(p);
                }
            }
            // Ignore ValuesMultiLines, Values::Lines,  Values::Points etc.
            _ => {
                panic!("unprocessed object");
            }
        }
    }

    /// Loop over the input pushing to internal state.
    /// `polygons_by_arc` and polygons.
    fn extract(&mut self, polygon: &[Vec<i32>]) {
        // Value to be stored and refered to .. in pba
        let pu = Rc::new(RefCell::new(PolygonU::from(polygon.to_vec())));

        for ring in polygon {
            for arc in ring {
                let index = translate(*arc);
                match self.polygons_by_arc.get_mut(&index) {
                    Some(p) => p.push(pu.clone()),
                    None => {
                        self.polygons_by_arc.insert(index, vec![pu.clone()]);
                    }
                };
            }
        }

        self.polygons.push(pu);
    }

    fn area(&self, ring: ArcIndexes) -> f64 {
        let polygon = Value::Polygon(vec![ring]);
        let object = feature(self.topology, &polygon);
        match object {
            Geometry::Polygon(p) => planar_ring_area(&p.exterior().0),
            _ => {
                todo!("was expecting a polygon");
            }
        }
    }
}

#[cfg(test)]
mod merge_tests {
    use geo::Geometry;
    use geo::LineString;
    use geo::MultiPolygon;
    use geo::Polygon;
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::Topology;
    use topojson::Value;

    use crate::merge::merge;

    #[test]
    fn merge_ignores_null_geometries() {
        println!("merge ignores null geometries");
        let topology = Topology {
            arcs: vec![],
            objects: vec![],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        let mp = Geometry::MultiPolygon(MultiPolygon::<f64>(vec![]));

        assert_eq!(merge(&topology, &[]), mp);
    }

    ///
    /// +----+----+            +----+----+
    /// |    |    |            |         |
    /// |    |    |    ==>     |         |
    /// |    |    |            |         |
    /// +----+----+            +----+----+
    ///
    #[test]
    fn stitches_together_two_side_by_side_polygons() {
        println!("merge stitches together two side-by-side polygons");

        let objects = vec![
            NamedGeometry {
                name: "a".to_string(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![
                    0, 1,
                ]])),
            },
            NamedGeometry {
                name: "b".to_string(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![
                    -1, 2,
                ]])),
            },
        ];

        let topology = Topology {
            arcs: vec![
                vec![vec![1_f64, 1_f64], vec![1_f64, 0_f64]],
                vec![
                    vec![1_f64, 0_f64],
                    vec![0_f64, 0_f64],
                    vec![0_f64, 1_f64],
                    vec![1_f64, 1_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                    vec![2_f64, 0_f64],
                    vec![1_f64, 0_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        let coords: Vec<(f64, f64)> = vec![
            (1_f64, 0_f64),
            (0_f64, 0_f64),
            (0_f64, 1_f64),
            (1_f64, 1_f64),
            (2_f64, 1_f64),
            (2_f64, 0_f64),
            (1_f64, 0_f64),
        ];
        let exterior: LineString<f64> = coords.into_iter().collect();
        let mp = Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
            exterior,
            vec![],
        )]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    //
    // +----+----+            +----+----+
    // |    |    |            |         |
    // |    |    |    ==>     |         |
    // |    |    |            |         |
    // +----+----+            +----+----+
    //
    #[test]
    fn stitches_together_geometry_collections() {
        println!("merge stitches together geometry collections");

        let polys = vec![
            topojson::Geometry::new(Value::Polygon(vec![vec![0, 1]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-1, 2]])),
        ];

        let object = Value::GeometryCollection(polys);

        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![vec![1_f64, 1_f64], vec![1_f64, 0_f64]],
                vec![
                    vec![1_f64, 0_f64],
                    vec![0_f64, 0_f64],
                    vec![0_f64, 1_f64],
                    vec![1_f64, 1_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                    vec![2_f64, 0_f64],
                    vec![1_f64, 0_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        let coords: Vec<(f64, f64)> = vec![
            (1_f64, 0_f64),
            (0_f64, 0_f64),
            (0_f64, 1_f64),
            (1_f64, 1_f64),
            (2_f64, 1_f64),
            (2_f64, 0_f64),
            (1_f64, 0_f64),
        ];
        let exterior: LineString<f64> = coords.into_iter().collect();
        let mp = Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
            exterior,
            vec![],
        )]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    #[test]
    fn merge_does_not_stitch_together_two_separated_polygons() {
        println!("merge does not stitch together two separated polygons");

        let objects = vec![
            NamedGeometry {
                name: "a".to_string(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![
                    0,
                ]])),
            },
            NamedGeometry {
                name: "b".to_string(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![
                    1,
                ]])),
            },
        ];

        let topology = Topology {
            arcs: vec![
                vec![
                    vec![0_f64, 0_f64],
                    vec![0_f64, 1_f64],
                    vec![1_f64, 1_f64],
                    vec![1_f64, 0_f64],
                    vec![0_f64, 0_f64],
                ],
                vec![
                    vec![2_f64, 0_f64],
                    vec![2_f64, 1_f64],
                    vec![3_f64, 1_f64],
                    vec![3_f64, 0_f64],
                    vec![2_f64, 0_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        let coords1: Vec<(f64, f64)> = vec![
            (0_f64, 0_f64),
            (0_f64, 1_f64),
            (1_f64, 1_f64),
            (1_f64, 0_f64),
            (0_f64, 0_f64),
        ];
        let coords2: Vec<(f64, f64)> = vec![
            (2_f64, 0_f64),
            (2_f64, 1_f64),
            (3_f64, 1_f64),
            (3_f64, 0_f64),
            (2_f64, 0_f64),
        ];
        let exterior1: LineString<f64> = coords1.into_iter().collect();
        let exterior2: LineString<f64> = coords2.into_iter().collect();
        let mp = Geometry::MultiPolygon(MultiPolygon(vec![
            Polygon::new(exterior1, vec![]),
            Polygon::new(exterior2, vec![]),
        ]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    //   //
    //   // +-----------+            +-----------+
    //   // |           |            |           |
    //   // |   +---+   |    ==>     |   +---+   |
    //   // |   |   |   |            |   |   |   |
    //   // |   +---+   |            |   +---+   |
    //   // |           |            |           |
    //   // +-----------+            +-----------+
    //   //
    #[test]
    fn merge_does_not_stitch_together_a_polygon_and_its_hole() {
        println!("merge does not stitch together a polygon and its hole");

        let polys = vec![topojson::Geometry::new(Value::Polygon(vec![
            vec![0],
            vec![1],
        ]))];
        let object = Value::GeometryCollection(polys);
        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![
                    vec![0_f64, 0_f64],
                    vec![0_f64, 3_f64],
                    vec![3_f64, 3_f64],
                    vec![3_f64, 0_f64],
                    vec![0_f64, 0_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                    vec![2_f64, 2_f64],
                    vec![1_f64, 2_f64],
                    vec![1_f64, 1_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };

        let p1 = Polygon::new(
            LineString::from(vec![
                (0.0_f64, 0.0_f64),
                (0.0_f64, 3.0_f64),
                (3.0_f64, 3.0_f64),
                (3.0_f64, 0.0_f64),
                (0.0_f64, 0.0_f64),
            ]),
            vec![LineString::from(vec![
                (1_f64, 1_f64),
                (2_f64, 1_f64),
                (2_f64, 2_f64),
                (1_f64, 2_f64),
                (1_f64, 1_f64),
            ])],
        );
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![p1]));

        assert_eq!(merge(&topology, &objects), mp);
    }
    //
    // +-----------+            +-----------+
    // |           |            |           |
    // |   +---+   |    ==>     |           |
    // |   |   |   |            |           |
    // |   +---+   |            |           |
    // |           |            |           |
    // +-----------+            +-----------+
    //
    #[test]
    fn merge_stitches_together_a_polygon_surrounding_another_polygon() {
        println!(
            "merge stitches together a polygon surrounding another polygon"
        );

        let polys = vec![
            topojson::Geometry::new(Value::Polygon(vec![vec![0], vec![1]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-2]])),
        ];
        let object = Value::GeometryCollection(polys);
        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![
                    vec![0_f64, 0_f64],
                    vec![0_f64, 3_f64],
                    vec![3_f64, 3_f64],
                    vec![3_f64, 0_f64],
                    vec![0_f64, 0_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                    vec![2_f64, 2_f64],
                    vec![1_f64, 2_f64],
                    vec![1_f64, 1_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };

        let p1 = Polygon::new(
            LineString::from(vec![
                (0.0_f64, 0.0_f64),
                (0.0_f64, 3.0_f64),
                (3.0_f64, 3.0_f64),
                (3.0_f64, 0.0_f64),
                (0.0_f64, 0.0_f64),
            ]),
            vec![],
        );
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![p1]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    //
    // +-----------+-----------+            +-----------+-----------+
    // |           |           |            |                       |
    // |   +---+   |   +---+   |    ==>     |   +---+       +---+   |
    // |   |   |   |   |   |   |            |   |   |       |   |   |
    // |   +---+   |   +---+   |            |   +---+       +---+   |
    // |           |           |            |                       |
    // +-----------+-----------+            +-----------+-----------+
    //
    #[test]
    fn stitches_together_two_side_by_side_polygons_with_holes() {
        println!(
            "merge stitches together two side-by-side polygons with holes"
        );

        let polys = vec![
            topojson::Geometry::new(Value::Polygon(vec![vec![0, 1], vec![2]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-1, 3], vec![4]])),
        ];
        let object = Value::GeometryCollection(polys);
        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![vec![3_f64, 3_f64], vec![3_f64, 0_f64]],
                vec![
                    vec![3_f64, 0_f64],
                    vec![0_f64, 0_f64],
                    vec![0_f64, 3_f64],
                    vec![3_f64, 3_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                    vec![2_f64, 2_f64],
                    vec![1_f64, 2_f64],
                    vec![1_f64, 1_f64],
                ],
                vec![
                    vec![3_f64, 3_f64],
                    vec![6_f64, 3_f64],
                    vec![6_f64, 0_f64],
                    vec![3_f64, 0_f64],
                ],
                vec![
                    vec![4_f64, 1_f64],
                    vec![5_f64, 1_f64],
                    vec![5_f64, 2_f64],
                    vec![4_f64, 2_f64],
                    vec![4_f64, 1_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };

        let p1 = Polygon::new(
            LineString::from(vec![
                (3.0_f64, 0.0_f64),
                (0.0_f64, 0.0_f64),
                (0.0_f64, 3.0_f64),
                (3.0_f64, 3.0_f64),
                (6.0_f64, 3.0_f64),
                (6.0_f64, 0.0_f64),
                (3.0_f64, 0.0_f64),
            ]),
            vec![
                LineString::from(vec![
                    (1., 1.),
                    (2., 1.),
                    (2., 2.),
                    (1., 2.),
                    (1., 1.),
                ]),
                LineString::from(vec![
                    (4., 1.),
                    (5., 1.),
                    (5., 2.),
                    (4., 2.),
                    (4., 1.),
                ]),
            ],
        );
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![p1]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    //
    // +-------+-------+            +-------+-------+
    // |       |       |            |               |
    // |   +---+---+   |    ==>     |   +---+---+   |
    // |   |       |   |            |   |       |   |
    // |   +---+---+   |            |   +---+---+   |
    // |       |       |            |               |
    // +-------+-------+            +-------+-------+
    //
    #[test]
    fn merge_stitches_together_two_horseshoe_polygons_creating_a_hole() {
        println!(
            "merge stitches together two horseshoe polygons, creating a hole"
        );

        let polys = vec![
            topojson::Geometry::new(Value::Polygon(vec![vec![0, 1, 2, 3]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-3, 4, -1, 5]])),
        ];
        let object = Value::GeometryCollection(polys);
        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![vec![2_f64, 3_f64], vec![2_f64, 2_f64]],
                vec![
                    vec![2_f64, 2_f64],
                    vec![1_f64, 2_f64],
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                ],
                vec![vec![2_f64, 1_f64], vec![2_f64, 0_f64]],
                vec![
                    vec![2_f64, 0_f64],
                    vec![0_f64, 0_f64],
                    vec![0_f64, 3_f64],
                    vec![2_f64, 3_f64],
                ],
                vec![
                    vec![2_f64, 1_f64],
                    vec![3_f64, 1_f64],
                    vec![3_f64, 2_f64],
                    vec![2_f64, 2_f64],
                ],
                vec![
                    vec![2_f64, 3_f64],
                    vec![4_f64, 3_f64],
                    vec![4_f64, 0_f64],
                    vec![2_f64, 0_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };

        let p1 = Polygon::new(
            LineString::from(vec![
                (2.0_f64, 0.0_f64),
                (0.0_f64, 0.0_f64),
                (0.0_f64, 3.0_f64),
                (2.0_f64, 3.0_f64),
                (4.0_f64, 3.0_f64),
                (4.0_f64, 0.0_f64),
                (2.0_f64, 0.0_f64),
            ]),
            vec![LineString::from(vec![
                (2., 2.),
                (1., 2.),
                (1., 1.),
                (2., 1.),
                (3., 1.),
                (3., 2.),
                (2., 2.),
            ])],
        );
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![p1]));

        assert_eq!(merge(&topology, &objects), mp);
    }

    //   //
    //   // +-------+-------+            +-------+-------+
    //   // |       |       |            |               |
    //   // |   +---+---+   |    ==>     |               |
    //   // |   |   |   |   |            |               |
    //   // |   +---+---+   |            |               |
    //   // |       |       |            |               |
    //   // +-------+-------+            +-------+-------+
    //   //
    //
    #[test]
    fn merge_stitches_together_two_horseshoe_polygons_surrounding_two_other_polygons(
    ) {
        println!("merge stitches together two horseshoe polygons surrounding two other polygons");

        let polys = vec![
            topojson::Geometry::new(Value::Polygon(vec![vec![0, 1, 2, 3]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-3, 4, -1, 5]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![6, -2]])),
            topojson::Geometry::new(Value::Polygon(vec![vec![-7, -5]])),
        ];
        let object = Value::GeometryCollection(polys);

        let objects = vec![NamedGeometry {
            name: "foo".to_string(),
            geometry: topojson::Geometry::new(object),
        }];

        let topology = Topology {
            arcs: vec![
                vec![vec![2_f64, 3_f64], vec![2_f64, 2_f64]],
                vec![
                    vec![2_f64, 2_f64],
                    vec![1_f64, 2_f64],
                    vec![1_f64, 1_f64],
                    vec![2_f64, 1_f64],
                ],
                vec![vec![2_f64, 1_f64], vec![2_f64, 0_f64]],
                vec![
                    vec![2_f64, 0_f64],
                    vec![0_f64, 0_f64],
                    vec![0_f64, 3_f64],
                    vec![2_f64, 3_f64],
                ],
                vec![
                    vec![2_f64, 1_f64],
                    vec![3_f64, 1_f64],
                    vec![3_f64, 2_f64],
                    vec![2_f64, 2_f64],
                ],
                vec![
                    vec![2_f64, 3_f64],
                    vec![4_f64, 3_f64],
                    vec![4_f64, 0_f64],
                    vec![2_f64, 0_f64],
                ],
            ],
            objects: objects.clone(),
            bbox: None,
            transform: None,
            foreign_members: None,
        };

        let p1 = Polygon::new(
            LineString::from(vec![
                (2.0_f64, 0.0_f64),
                (0.0_f64, 0.0_f64),
                (0.0_f64, 3.0_f64),
                (2.0_f64, 3.0_f64),
                (4.0_f64, 3.0_f64),
                (4.0_f64, 0.0_f64),
                (2.0_f64, 0.0_f64),
            ]),
            vec![],
        );
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![p1]));

        assert_eq!(merge(&topology, &objects), mp);
    }
}
