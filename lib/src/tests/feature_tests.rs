#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod feature_tests {

    use super::*;
    use geo::Coordinate;
    use geo::Geometry;
    use geo::GeometryCollection;
    use geo::LineString;
    use geo::MultiLineString;
    use geo::MultiPolygon;
    use geo::Point;
    use geo::Polygon;
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::TransformParams;
    use topojson::Value;

    #[test]
    fn geometry_type_is_preserved() {
        println!("topojson.feature the geometry type is preserved");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![0]])));

        match feature_from_name::<f64>(&t, &"foo") {
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
        let t = simple_topology(topojson::Geometry::new(Value::Point(vec![0_f64, 0_f64])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::Point(Point(Coordinate { x: 0_f64, y: 0_f64 })))
        );
    }

    #[test]
    fn multipoint() {
        println!("topojson.feature MultiPoint is a valid geometry type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiPoint(vec![
            vec![0_f64, 0_f64],
            vec![0xf0 as f64, 0xba as f64],
        ])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::MultiPoint(MultiPoint(vec![
                Point(Coordinate { x: 0_f64, y: 0_f64 }),
                Point(Coordinate {
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
        let t = simple_topology(topojson::Geometry::new(Value::LineString(vec![0])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::LineString(LineString(vec![
                Coordinate { x: 0_f64, y: 0_f64 },
                Coordinate { x: 1_f64, y: 0_f64 },
                Coordinate { x: 1_f64, y: 1_f64 },
                Coordinate { x: 0_f64, y: 1_f64 },
                Coordinate { x: 0_f64, y: 0_f64 },
            ])))
        );
    }

    #[test]
    fn multiline_string() {
        println!("topojson.feature MultiLineString is a valid geometry type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiLineString(vec![vec![
            0,
        ]])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::MultiLineString(MultiLineString(vec![
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ])
            ])))
        );
    }

    #[test]
    fn line_string_two_coords() {
        println!("topojson.feature line-strings have at least two coordinates");
        let t1 = simple_topology(topojson::Geometry::new(Value::LineString(vec![3])));

        assert_eq!(
            feature_from_name(&t1, &"foo"),
            Some(Geometry::LineString(LineString(vec![
                Coordinate { x: 1_f64, y: 1_f64 },
                Coordinate { x: 1_f64, y: 1_f64 },
            ])))
        );

        let t2 = simple_topology(topojson::Geometry::new(Value::MultiLineString(vec![
            vec![3],
            vec![4],
        ])));

        assert_eq!(
            feature_from_name(&t2, &"foo"),
            Some(Geometry::MultiLineString(MultiLineString(vec![
                LineString(vec![
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                ]),
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
            ])))
        );
    }

    #[test]
    fn polygon() {
        println!("topojson.feature Polygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![0]])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn multipolygon() {
        println!("topojson.feature MultiPolygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiPolygon(vec![vec![
            vec![0],
        ]])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
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
                    geometry: topojson::Geometry::new(Value::Polygon(vec![vec![0]])),
                },
                NamedGeometry {
                    name: "bar".to_string(),
                    geometry: topojson::Geometry::new(Value::Polygon(vec![vec![0, 1]])),
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
            feature_from_name(&topology, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 }
                ]),
                vec![]
            )))
        );

        assert_eq!(
            feature_from_name(&topology, &"bar"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 }
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

        let t = simple_topology(topojson::Geometry::new(Value::GeometryCollection(vec![
            topojson::Geometry::new(Value::MultiPolygon(vec![vec![vec![0]]])),
        ])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::GeometryCollection(GeometryCollection(vec![
                Geometry::MultiPolygon(MultiPolygon(vec![Polygon::new(
                    LineString(vec![
                        Coordinate { x: 0.0, y: 0.0 },
                        Coordinate { x: 1.0, y: 0.0 },
                        Coordinate { x: 1.0, y: 1.0 },
                        Coordinate { x: 0.0, y: 1.0 },
                        Coordinate { x: 0.0, y: 0.0 },
                    ]),
                    vec![]
                )]))
            ])))
        );
    }

    #[test]
    fn gc_nested() {
        println!("topojson.feature geometry collections can be nested",);

        let t = simple_topology(topojson::Geometry::new(Value::GeometryCollection(vec![
            topojson::Geometry::new(Value::GeometryCollection(vec![topojson::Geometry::new(
                Value::Point(vec![0_f64, 0_f64]),
            )])),
        ])));

        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::GeometryCollection(GeometryCollection(vec![
                Geometry::GeometryCollection(GeometryCollection(vec![Geometry::Point(Point(
                    Coordinate { x: 0_f64, y: 0_f64 }
                ))]))
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
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![0_i32]])));
        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn negative_indexes_indicates_revered_coordinates() {
        println!("topojson.feature negative arc indexes indicate reversed coordinates");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![!0_i32]])));
        assert_eq!(
            feature_from_name(&t, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
                vec![]
            )))
        );
    }

    #[test]
    fn when_multiple_arc_indexes_are_specified_coordinates_are_stitched_together() {
        println!("topojson.feature when multiple arc indexes are specified, coordinates are stitched together");
        let t1 = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![
            1_i32, 2_i32,
        ]])));
        assert_eq!(
            feature_from_name(&t1, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
                vec![],
            )))
        );

        let t2 = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![
            !2_i32, !1_i32,
        ]])));
        assert_eq!(
            feature_from_name(&t2, &"foo"),
            Some(Geometry::Polygon(Polygon::new(
                LineString(vec![
                    Coordinate { x: 0_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 1_f64 },
                    Coordinate { x: 1_f64, y: 1_f64 },
                    Coordinate { x: 1_f64, y: 0_f64 },
                    Coordinate { x: 0_f64, y: 0_f64 },
                ]),
                vec![],
            )))
        )
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
                geometry: topojson::Geometry::new(Value::Point(vec![1_f64, 2_f64])),
            }],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        assert_eq!(
            feature_from_name(&t, &"foo"),
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
                geometry: topojson::Geometry::new(Value::MultiPoint(vec![vec![1_f64, 2_f64]])),
            }],
            bbox: None,
            transform: None,
            foreign_members: None,
        };
        assert_eq!(
            feature_from_name(&t, &"foo"),
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
            feature_from_name(&t, &"foo"),
            Some(Geometry::LineString(LineString(vec![
                Coordinate { x: 1_f64, y: 2_f64 },
                Coordinate { x: 3_f64, y: 4_f64 }
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
                vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64], vec![0_f64, 1_f64]],
                vec![vec![1_f64, 1_f64], vec![-1_f64, 0_f64], vec![0_f64, -1_f64]],
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
