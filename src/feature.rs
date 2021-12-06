use std::marker::PhantomData;
use std::ops::AddAssign;

use derivative::*;
use geo::CoordFloat;
use geo::Coordinate;
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
use crate::transform::generate as transform;
use crate::transform::TransformFn;

#[derive(Derivative)]
#[derivative(Debug)]
/// State holds data extracted from a Topological object.
pub struct Builder<T>
where
    T: AddAssign<T> + CoordFloat,
{
    pd: PhantomData<T>,
    arcs: Vec<Arc>,
    #[derivative(Debug = "ignore")]
    transform_point: TransformFn<T>,
}

impl<T> Builder<T>
where
    T: 'static + AddAssign<T> + CoordFloat,
{
    /// Given a object name find convert and return a Geometry object.
    ///
    /// None: -
    ///   * The topology tranform is ommitted.
    ///   * The object subsection does not contain the name.
    #[inline]
    pub fn generate_from_name(topology: &Topology, name: &str) -> Option<Geometry<T>> {
        match topology.objects.iter().find(|x| x.name == name) {
            Some(ng) => Builder::generate(topology, &ng.geometry.value),
            None => None,
        }
    }

    /// Given a json gemetry value apply a transform and convert.
    ///
    /// None: -
    ///     The topology transform is ommitted.
    #[inline]
    pub fn generate(topology: &Topology, o: &Value) -> Option<Geometry<T>>
    where
        T: 'static + CoordFloat,
    {
        match &topology.transform {
            None => None,
            Some(transform_params) => {
                let transform = transform::<T>(transform_params);
                let mut out = Self {
                    pd: PhantomData::<T>::default(),
                    arcs: topology.arcs.clone(),
                    transform_point: transform,
                };

                Some(out.geometry(o))
            }
        }
    }

    /// Convert the index found in a Geometry object into a point.
    ///
    /// Using the top level arcs array as reference.
    fn arc(&mut self, i: i32, points: &mut Vec<(T, T)>) {
        if !points.is_empty() {
            points.pop();
        }

        // As per spec. negative indicies are bit wise NOT converted.
        let index = if i < 0 { !(i) } else { i } as usize;
        let a = &self.arcs[index];
        let n = a.len();
        for (k, v) in a.iter().enumerate() {
            let t = (self.transform_point)(v, k);
            points.push((t[0], t[1]));
        }

        if i < 0 {
            reverse(points, n);
        }
    }

    /// Transform a single point.
    #[inline]
    fn point(&mut self, p: &[f64]) -> Vec<T> {
        (self.transform_point)(p, 0)
    }

    /// Convert a array of indicies found in a Geometry object into a arrays of
    /// points.
    ///
    /// Using the top level arcs array as reference.
    fn line(&mut self, arcs: &[i32]) -> Vec<(T, T)> {
        let mut points: Vec<(T, T)> = Vec::with_capacity(arcs.len() + 1);
        for a in arcs {
            self.arc(*a, &mut points);
        }

        if points.len() < 2 {
            // This should never happen per the specification.
            points.push(points[0]);
        }

        points
    }

    fn ring(&mut self, arcs: &[i32]) -> Vec<(T, T)> {
        let mut points = self.line(arcs);
        // This may happen if an arc has only two points.
        while points.len() < 4 {
            points.push(points[0]);
        }
        points
    }

    #[inline]
    fn polygon(&mut self, arcs: &[ArcIndexes]) -> Vec<Vec<(T, T)>> {
        arcs.iter().map(|x| self.ring(x)).collect()
    }

    #[inline]
    fn geometry(&mut self, o: &Value) -> Geometry<T> {
        match &o {
            Value::GeometryCollection(topo_geometries) => {
                let geo_geometries: Vec<Geometry<T>> = topo_geometries
                    .iter()
                    .map(|x| self.geometry(&x.value))
                    .collect();
                Geometry::GeometryCollection(GeometryCollection(geo_geometries))
            }
            Value::Point(topo_point) => {
                let p = self.point(topo_point);
                Geometry::Point(Point(Coordinate::<T> { x: p[0], y: p[1] }))
            }
            Value::MultiPoint(topo_multipoint) => {
                let points: Vec<Point<T>> = topo_multipoint
                    .iter()
                    .map(|c| {
                        let p = self.point(c);
                        Point(Coordinate::<T> { x: p[0], y: p[1] })
                    })
                    .collect();
                let geo_multipoint: MultiPoint<T> = MultiPoint(points);
                Geometry::MultiPoint(geo_multipoint)
            }
            Value::LineString(topo_ls) => {
                // self.arc(0, topo_ls);
                let line = self.line(topo_ls);
                let geo_ls: LineString<T> = line.into();
                Geometry::LineString(geo_ls)
            }
            Value::MultiLineString(topo_mls) => {
                let v_mls: Vec<LineString<T>> =
                    topo_mls.iter().map(|x| self.line(x).into()).collect();
                Geometry::MultiLineString(MultiLineString(v_mls))
            }
            Value::Polygon(topo_polygon) => {
                let v_linestring: Vec<LineString<T>> = self
                    .polygon(topo_polygon)
                    .iter()
                    .map(|x| {
                        let x1: Vec<(T, T)> = (*x).iter().copied().collect();
                        let tmp: LineString<T> = x1.into();
                        tmp
                    })
                    .collect();
                let exterior: LineString<T> = v_linestring[0].clone();
                let interior = v_linestring[1..].to_vec();
                Geometry::Polygon(Polygon::new(exterior, interior))
            }
            Value::MultiPolygon(topo_mp) => {
                let v_polygon: Vec<Polygon<T>> = topo_mp
                    .iter()
                    .map(|x| {
                        let v_linestring: Vec<LineString<T>> =
                            self.polygon(x).iter().map(|y| (y.clone()).into()).collect();

                        let exterior = v_linestring[0].clone();
                        let interior: Vec<LineString<T>> = v_linestring[1..].to_vec();
                        Polygon::new(exterior, interior)
                    })
                    .collect();

                Geometry::MultiPolygon(MultiPolygon(v_polygon))
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {

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
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        match computed {
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
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
            Some(Geometry::Point(Point(Coordinate { x: 0_f64, y: 0_f64 })))
        );
    }

    #[test]
    fn multipoint() {
        println!("topojson.feature MultiPoint is a valid geometry type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiPoint(vec![vec![
            0_f64, 0_f64,
        ]])));
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
            Some(Geometry::MultiPoint(MultiPoint(vec![Point(Coordinate {
                x: 0_f64,
                y: 0_f64
            })])))
        );
    }

    #[test]
    fn linestring() {
        println!("topojson.feature LineString is a valid geometry type");
        // TODO javascript test supplied arc indexes not arrays of points.
        let t = simple_topology(topojson::Geometry::new(Value::LineString(vec![0])));
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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
        let computed1: Option<Geometry<f64>> = Builder::<f64>::generate_from_name(&t1, &"foo");

        assert_eq!(
            computed1,
            Some(Geometry::LineString(LineString(vec![
                Coordinate { x: 1_f64, y: 1_f64 },
                Coordinate { x: 1_f64, y: 1_f64 },
            ])))
        );

        let t2 = simple_topology(topojson::Geometry::new(Value::MultiLineString(vec![
            vec![3],
            vec![4],
        ])));
        let computed2 = Builder::<f64>::generate_from_name(&t2, &"foo");

        assert_eq!(
            computed2,
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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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
        println!("topojson.feature Polygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::MultiPolygon(vec![vec![
            vec![0],
        ]])));
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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
        let computed: Option<Geometry<f64>> = Builder::generate_from_name(&topology, &"foo");
        assert_eq!(
            computed,
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

        let computed: Option<Geometry<f64>> = Builder::generate_from_name(&topology, &"bar");
        assert_eq!(
            computed,
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

        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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

        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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

    //   tape("topojson.feature arcs are converted to coordinates", function(test) {
    //     var t = simpleTopology({type: "Polygon", arcs: [[0]]});
    //     test.deepEqual(topojson.feature(t, t.objects.foo).geometry.coordinates, [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]);
    //     test.end();
    //   });

    #[test]
    fn negative_indexes_indicates_revered_coordinates() {
        println!("topojson.feature Polygon is a valid feature type");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![!0_i32]])));
        let computed = Builder::<f64>::generate_from_name(&t, &"foo");

        assert_eq!(
            computed,
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

    // TODO missing tests

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
