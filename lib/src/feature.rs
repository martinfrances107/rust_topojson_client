use geo::line_string;
use geo::Coord;
use geo::CoordFloat;
use geo::Geometry;
use geo::GeometryCollection;
use geo::LineString;
use geo::MultiLineString;
use geo::MultiPoint;
use geo::MultiPolygon;
use geo::Point;
use geo::Polygon;
use topojson::Arc;
use topojson::ArcIndexes;
use topojson::Topology;
use topojson::Value;

use crate::reverse::reverse;
use crate::transform::gen_transform;
use crate::transform::Transform;

/// Given a object name find convert and return a Geometry object.
///
/// None: -
///   * The object subsection does not contain the name.
#[inline]
#[must_use]
pub fn feature_from_name<T>(
    topology: &Topology,
    name: &str,
) -> Option<Geometry<T>>
where
    T: CoordFloat,
{
    topology
        .objects
        .iter()
        .find(|x| x.name == name)
        .map(|ng| feature(topology, &ng.geometry.value))
}

/// Given a json gemetry value apply a transform and convert.
///
/// If the transform params are ommited use identity scaling
/// and no translation.
#[inline]
#[must_use]
pub fn feature<T>(topology: &Topology, o: &Value) -> Geometry<T>
where
    T: CoordFloat,
{
    // let tp = match &topology.transform {
    //     None => &TransformParams {
    //         scale: [1_f64, 1_f64],
    //         translate: [0_f64, 0_f64],
    //     },
    //     Some(transform_params) => transform_params,
    // };

    Builder {
        arcs: topology.arcs.clone(),
        transform: gen_transform(&topology.transform),
    }
    .geometry(o)
}

/// State holds data extracted from a Topological object.
struct Builder {
    arcs: Vec<Arc>,

    transform: Transform,
}

impl Builder {
    /// Convert the index found in a Geometry object into a point.
    ///
    /// Using the top level arcs array as reference.
    fn arc(&mut self, i: i32, points: &mut Vec<(f64, f64)>) {
        if !points.is_empty() {
            points.pop();
        }

        // As per spec. negative indicies are bit wise NOT converted.
        let index = if i < 0 { !i } else { i } as usize;
        let a = &self.arcs[index];
        let n = a.len();
        for (k, v) in a.iter().enumerate() {
            let t = (self.transform)(v, k);
            points.push((t[0], t[1]));
        }

        if i < 0 {
            reverse(points, n);
        }
    }

    /// Transform a single point.
    #[inline]
    fn point(&mut self, p: &[f64]) -> Vec<f64> {
        (self.transform)(p, 0)
    }

    /// Convert a array of indicies found in a Geometry object into a arrays of
    /// points.
    ///
    /// Using the top level arcs array as reference.
    fn line(&mut self, arcs: &[i32]) -> Vec<(f64, f64)> {
        let mut points: Vec<(f64, f64)> = Vec::with_capacity(arcs.len() + 1);
        for a in arcs {
            self.arc(*a, &mut points);
        }

        if points.len() < 2 {
            // This should never happen per the specification.
            points.push(points[0]);
        }

        points
    }

    fn ring(&mut self, arcs: &[i32]) -> Vec<(f64, f64)> {
        let mut points = self.line(arcs);
        // This may happen if an arc has only two points.
        while points.len() < 4 {
            points.push(points[0]);
        }
        points
    }

    #[inline]
    fn polygon<'a>(
        &'a mut self,
        arcs: &'a [ArcIndexes],
    ) -> impl Iterator<Item = LineString<f64>> + 'a {
        arcs.iter().map(move |x| self.ring(x)).map(|x| {
            let x1: Vec<(f64, f64)> = (*x).to_vec();
            let mut tmp: LineString<f64> = x1.into();
            tmp.close();
            tmp
        })
    }

    /// For collections recursively build objects.
    #[inline]
    fn geometry<T>(&mut self, o: &Value) -> Geometry<T>
    where
        T: CoordFloat,
    {
        match &o {
            Value::GeometryCollection(topo_geometries) => {
                let geometries: Vec<Geometry<T>> = topo_geometries
                    .iter()
                    .map(|x| self.geometry(&x.value))
                    .collect();
                Geometry::GeometryCollection(GeometryCollection(geometries))
            }
            Value::Point(topo_point) => {
                let p = self.point(topo_point);
                Geometry::Point(Point(Coord::<T> {
                    x: T::from(p[0]).unwrap(),
                    y: T::from(p[1]).unwrap(),
                }))
            }
            Value::MultiPoint(topo_multipoint) => {
                let points: Vec<Point<T>> = topo_multipoint
                    .iter()
                    .map(|c| {
                        let p = self.point(c);
                        Point(Coord::<T> {
                            x: T::from(p[0]).unwrap(),
                            y: T::from(p[1]).unwrap(),
                        })
                    })
                    .collect();
                let multipoint = MultiPoint(points);
                Geometry::MultiPoint(multipoint)
            }
            Value::LineString(topo_ls) => {
                let line = self.line(topo_ls);
                let ls: LineString<T> = line
                    .iter()
                    .map(|p| Coord {
                        x: T::from(p.0).unwrap(),
                        y: T::from(p.1).unwrap(),
                    })
                    .collect();
                Geometry::LineString(ls)
            }

            Value::MultiLineString(topo_mls) => {
                let mls: Vec<LineString<T>> = topo_mls
                    .iter()
                    .map(|x| self.line(x))
                    .map(|vec| {
                        vec.iter()
                            .map(|p| Coord {
                                x: T::from(p.0).unwrap(),
                                y: T::from(p.1).unwrap(),
                            })
                            .collect()
                    })
                    .collect();
                Geometry::MultiLineString(MultiLineString(mls))
            }
            Value::Polygon(topo_polygon) => {
                let mut linestring_iter = self.polygon(topo_polygon);
                match linestring_iter.next() {
                    Some(exterior) => {
                        let interior = linestring_iter
                            .map(|ls| {
                                ls.0.iter()
                                    .map(|p| Coord {
                                        x: T::from(p.x).unwrap(),
                                        y: T::from(p.y).unwrap(),
                                    })
                                    .collect()
                            })
                            .collect();
                        let exterior = exterior
                            .0
                            .iter()
                            .map(|p| Coord {
                                x: T::from(p.x).unwrap(),
                                y: T::from(p.y).unwrap(),
                            })
                            .collect();
                        Geometry::Polygon(Polygon::new(exterior, interior))
                    }
                    None => {
                        Geometry::Polygon(Polygon::new(line_string![], vec![]))
                    }
                }
            }
            Value::MultiPolygon(topo_mp) => {
                let polygon: Vec<Polygon<T>> = topo_mp
                    .iter()
                    .map(|topo_polygon| {
                        let mut linestring_iter = self.polygon(topo_polygon);
                        match linestring_iter.next() {
                            Some(exterior) => {
                                let exterior = exterior
                                    .0
                                    .iter()
                                    .map(|p| Coord {
                                        x: T::from(p.x).unwrap(),
                                        y: T::from(p.y).unwrap(),
                                    })
                                    .collect();
                                let interior = linestring_iter
                                    .map(|ls| {
                                        ls.0.iter()
                                            .map(|p| Coord {
                                                x: T::from(p.x).unwrap(),
                                                y: T::from(p.y).unwrap(),
                                            })
                                            .collect()
                                    })
                                    .collect();
                                Polygon::new(exterior, interior)
                            }
                            None => Polygon::new(line_string![], vec![]),
                        }
                    })
                    .collect();

                Geometry::MultiPolygon(MultiPolygon(polygon))
            }
        }
    }
}

#[cfg(test)]
mod feature_tests {

    use super::*;
    use geo::Geometry;
    use geo::GeometryCollection;
    use geo::LineString;
    use geo::MultiLineString;
    use geo::MultiPolygon;
    use geo::Point;
    use geo::Polygon;
    use geo_types::Coord;
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::TransformParams;
    use topojson::Value;

    #[test]
    fn geometry_type_is_preserved() {
        println!("topojson.feature the geometry type is preserved");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![
            vec![0],
        ])));

        match feature_from_name::<f64>(&t, "foo") {
            Some(g) => match g {
                Geometry::Polygon(_) => {
                    assert!(true, "Must produce polygon");
                }
                _ => {
                    assert!(false, "did not decode to a polygon");
                }
            },
            None => {
                assert!(false, "should have returned with geometry");
            }
        };
    }

    #[test]
    fn point() {
        println!("topojson.feature Point is a valid geometry type");
        let t = simple_topology(topojson::Geometry::new(Value::Point(vec![
            0_f64, 0_f64,
        ])));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::Point(Point(Coord { x: 0_f64, y: 0_f64 })))
        );
    }

    #[test]
    fn multipoint() {
        println!("topojson.feature MultiPoint is a valid geometry type");
        let t =
            simple_topology(topojson::Geometry::new(Value::MultiPoint(vec![
                vec![0_f64, 0_f64],
                vec![0xf0 as f64, 0xba as f64],
            ])));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::MultiPoint(MultiPoint(vec![
                Point(Coord { x: 0_f64, y: 0_f64 }),
                Point(Coord {
                    x: 0xf0 as f64,
                    y: 0xba as f64
                })
            ])))
        );
    }

    #[test]
    fn linestring() {
        println!("topojson.feature LineString is a valid geometry type");
        // TODO javascript test supplied arc indexes not arrays of points.
        let t =
            simple_topology(topojson::Geometry::new(Value::LineString(vec![
                0,
            ])));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::LineString(LineString(vec![
                Coord { x: 0_f64, y: 0_f64 },
                Coord { x: 1_f64, y: 0_f64 },
                Coord { x: 1_f64, y: 1_f64 },
                Coord { x: 0_f64, y: 1_f64 },
                Coord { x: 0_f64, y: 0_f64 },
            ])))
        );
    }

    #[test]
    fn multiline_string() {
        println!("topojson.feature MultiLineString is a valid geometry type");
        let t = simple_topology(topojson::Geometry::new(
            Value::MultiLineString(vec![vec![0]]),
        ));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::MultiLineString(MultiLineString(vec![
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ])
            ])))
        );
    }

    #[test]
    fn line_string_two_coords() {
        println!("topojson.feature line-strings have at least two coordinates");
        let t1 =
            simple_topology(topojson::Geometry::new(Value::LineString(vec![
                3,
            ])));

        assert_eq!(
            feature_from_name(&t1, "foo"),
            Some(Geometry::LineString(LineString(vec![
                Coord { x: 1_f64, y: 1_f64 },
                Coord { x: 1_f64, y: 1_f64 },
            ])))
        );

        let t2 = simple_topology(topojson::Geometry::new(
            Value::MultiLineString(vec![vec![3], vec![4]]),
        ));

        assert_eq!(
            feature_from_name(&t2, "foo"),
            Some(Geometry::MultiLineString(MultiLineString(vec![
                LineString(vec![
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                ]),
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
            ])))
        );
    }

    #[test]
    fn polygon() {
        println!("topojson.feature Polygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![
            vec![0],
        ])));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn multipolygon() {
        println!("topojson.feature MultiPolygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiPolygon(
            vec![vec![vec![0]]],
        )));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )])))
        );
    }

    #[test]
    fn polygons_are_closed_with_at_least_four_coordinates() {
        println!("topojson.feature polygons are closed, with at least four coordinates");
        let topology = Topology {
            bbox: None,
            objects: vec![
                NamedGeometry {
                    name: "foo".to_string(),
                    geometry: topojson::Geometry::new(Value::Polygon(vec![
                        vec![0],
                    ])),
                },
                NamedGeometry {
                    name: "bar".to_string(),
                    geometry: topojson::Geometry::new(Value::Polygon(vec![
                        vec![0, 1],
                    ])),
                },
            ],
            transform: Some(TransformParams {
                scale: [1_f64, 1_f64],
                translate: [0_f64, 0_f64],
            }),
            arcs: vec![
                vec![vec![0_f64, 0_f64], vec![1_f64, 1_f64]],
                vec![vec![1_f64, 1_f64], vec![-1_f64, -1_f64]],
            ],
            foreign_members: None,
        };

        assert_eq!(
            feature_from_name(&topology, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 0_f64 }
                ]),
                vec![]
            )))
        );

        assert_eq!(
            feature_from_name(&topology, "bar"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 0_f64 }
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn gc_are_mapped_to_fc() {
        println!(
            "topojson.feature top-level geometry collections are mapped to feature collections"
        );

        let t = simple_topology(topojson::Geometry::new(
            Value::GeometryCollection(vec![topojson::Geometry::new(
                Value::MultiPolygon(vec![vec![vec![0]]]),
            )]),
        ));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::GeometryCollection(GeometryCollection(vec![
                Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
                    LineString(vec![
                        Coord { x: 0.0, y: 0.0 },
                        Coord { x: 1.0, y: 0.0 },
                        Coord { x: 1.0, y: 1.0 },
                        Coord { x: 0.0, y: 1.0 },
                        Coord { x: 0.0, y: 0.0 },
                    ]),
                    vec![]
                )]))
            ])))
        );
    }

    #[test]
    fn gc_nested() {
        println!("topojson.feature geometry collections can be nested",);

        let t = simple_topology(topojson::Geometry::new(
            Value::GeometryCollection(vec![topojson::Geometry::new(
                Value::GeometryCollection(vec![topojson::Geometry::new(
                    Value::Point(vec![0_f64, 0_f64]),
                )]),
            )]),
        ));

        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::GeometryCollection(GeometryCollection(vec![
                Geometry::GeometryCollection(GeometryCollection(vec![
                    Geometry::Point(Point(Coord { x: 0_f64, y: 0_f64 }))
                ]))
            ])))
        );
    }

    // tape("topojson.feature top-level geometry collections do not have ids, but second-level geometry collections can", function(test) {
    //     var t = simpleTopology({type: "GeometryCollection", id: "collection", geometries: [{type: "GeometryCollection", id: "feature", geometries: [{type: "Point", id: "geometry", coordinates: [0, 0]}]}]});
    //     test.deepEqual(topojson.feature(t, t.objects.foo), {type: "FeatureCollection", features: [{type: "Feature", id: "feature", properties: {}, geometry: {type: "GeometryCollection", geometries: [{type: "Point", coordinates: [0, 0]}]}}]});
    //     test.end();
    //   });

    //   tape("topojson.feature top-level geometry collections do not have properties, but second-level geometry collections can", function(test) {
    //     var t = simpleTopology({type: "GeometryCollection", properties: {collection: true}, geometries: [{type: "GeometryCollection", properties: {feature: true}, geometries: [{type: "Point", properties: {geometry: true}, coordinates: [0, 0]}]}]});
    //     test.deepEqual(topojson.feature(t, t.objects.foo), {type: "FeatureCollection", features: [{type: "Feature", properties: {feature: true}, geometry: {type: "GeometryCollection", geometries: [{type: "Point", coordinates: [0, 0]}]}}]});
    //     test.end();
    //   });

    //   tape("topojson.feature the object id is promoted to feature id", function(test) {
    //     var t = simpleTopology({id: "foo", type: "Polygon", arcs: [[0]]});
    //     test.equal(topojson.feature(t, t.objects.foo).id, "foo");
    //     test.end();
    //   });

    //   tape("topojson.feature any object properties are promoted to feature properties", function(test) {
    //     var t = simpleTopology({type: "Polygon", properties: {color: "orange", size: 42}, arcs: [[0]]});
    //     test.deepEqual(topojson.feature(t, t.objects.foo).properties, {color: "orange", size: 42});
    //     test.end();
    //   });

    //   tape("topojson.feature the object id is optional", function(test) {
    //     var t = simpleTopology({type: "Polygon", arcs: [[0]]});
    //     test.equal(topojson.feature(t, t.objects.foo).id, undefined);
    //     test.end();
    //   });

    //   tape("topojson.feature object properties are created if missing", function(test) {
    //     var t = simpleTopology({type: "Polygon", arcs: [[0]]});
    //     test.deepEqual(topojson.feature(t, t.objects.foo).properties, {});
    //     test.end();
    //   });

    #[test]
    fn arcs_are_converted_coordinates() {
        println!("topojson.feature arcs are converted to coordinates");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![
            vec![0_i32],
        ])));
        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn negative_indexes_indicates_revered_coordinates() {
        println!("topojson.feature negative arc indexes indicate reversed coordinates");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![
            vec![!0_i32],
        ])));
        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn when_multiple_arc_indexes_are_specified_coordinates_are_stitched_together(
    ) {
        println!("topojson.feature when multiple arc indexes are specified, coordinates are stitched together");
        let t1 =
            simple_topology(topojson::Geometry::new(Value::Polygon(vec![
                vec![1_i32, 2_i32],
            ])));
        assert_eq!(
            feature_from_name(&t1, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![],
            )))
        );

        let t2 =
            simple_topology(topojson::Geometry::new(Value::Polygon(vec![
                vec![!2_i32, !1_i32],
            ])));
        assert_eq!(
            feature_from_name(&t2, "foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coord { x: 0_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 1_f64 },
                    Coord { x: 1_f64, y: 1_f64 },
                    Coord { x: 1_f64, y: 0_f64 },
                    Coord { x: 0_f64, y: 0_f64 },
                ]),
                vec![],
            )))
        );
    }

    // Cannot port this test are Geometry types "Unknown" is not possible in rust.
    // everything must have a concrete type.
    //
    //   tape("topojson.feature unknown geometry types are converted to null geometries", function (test) {
    //     var topology = {
    //       type: "Topology",
    //       transform: { scale: [1, 1], translate: [0, 0] },
    //       objects: {
    //         foo: { id: "foo" },
    //         bar: { type: "Invalid", properties: { bar: 2 } },
    //         baz: { type: "GeometryCollection", geometries: [{ type: "Unknown", id: "unknown" }] }
    //       },
    //       arcs: []
    //     };
    //     test.deepEqual(topojson.feature(topology, topology.objects.foo), { type: "Feature", id: "foo", properties: {}, geometry: null });
    //     test.deepEqual(topojson.feature(topology, topology.objects.bar), { type: "Feature", properties: { bar: 2 }, geometry: null });
    //     test.deepEqual(topojson.feature(topology, topology.objects.baz), { type: "FeatureCollection", features: [{ type: "Feature", id: "unknown", properties: {}, geometry: null }] });
    //     test.end();
    //   });

    #[test]
    fn preserves_additional_dimensions_in_point_geometries() {
        println!("topojson.feature preserves additional dimensions in Point geometries");
        let t = Topology {
            arcs: vec![],
            objects: vec![NamedGeometry {
                name: "foo".to_string(),
                geometry: topojson::Geometry::new(Value::Point(vec![
                    1_f64, 2_f64,
                ])),
            }],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::Point(Point::new(1_f64, 2_f64)))
        );
    }

    #[test]
    fn preserves_additional_dimensions_in_multipoint_geometries() {
        println!("topojson.feature preserves additional dimensions in MultiPoint geometries");
        let t = Topology {
            arcs: vec![],
            objects: vec![NamedGeometry {
                name: "foo".to_string(),
                geometry: topojson::Geometry::new(Value::MultiPoint(vec![
                    vec![1_f64, 2_f64],
                ])),
            }],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::MultiPoint(MultiPoint(vec![Point::new(
                1_f64, 2_f64
            )])))
        );
    }

    #[test]
    fn preserves_additional_dimensions_in_linestring_geometries() {
        println!("topojson.feature preserves additional dimensions in LineString geometries");
        let t = Topology {
            arcs: vec![vec![
                vec![1_f64, 2_f64, 0xf00 as f64, 0xbe as f64],
                vec![3_f64, 4_f64, 0xbae as f64, 0xef as f64],
            ]],
            objects: vec![NamedGeometry {
                name: "foo".to_string(),
                geometry: topojson::Geometry::new(Value::LineString(vec![0])),
            }],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        assert_eq!(
            feature_from_name(&t, "foo"),
            Some(Geometry::LineString(LineString(vec![
                Coord { x: 1_f64, y: 2_f64 },
                Coord { x: 3_f64, y: 4_f64 }
            ])))
        );
    }
    fn simple_topology(object: topojson::Geometry) -> Topology {
        Topology {
            arcs: vec![
                vec![
                    vec![0_f64, 0_f64],
                    vec![1_f64, 0_f64],
                    vec![0_f64, 1_f64],
                    vec![-1_f64, 0_f64],
                    vec![0_f64, -1_f64],
                ],
                vec![
                    vec![0_f64, 0_f64],
                    vec![1_f64, 0_f64],
                    vec![0_f64, 1_f64],
                ],
                vec![
                    vec![1_f64, 1_f64],
                    vec![-1_f64, 0_f64],
                    vec![0_f64, -1_f64],
                ],
                vec![vec![1_f64, 1_f64]],
                vec![vec![0_f64, 0_f64]],
            ],
            objects: vec![NamedGeometry {
                name: "foo".to_string(),
                geometry: object,
            }],
            bbox: None,
            transform: Some(TransformParams {
                scale: [1_f64, 1_f64],
                translate: [0_f64, 0_f64],
            }),
            foreign_members: None,
        }
    }
}
