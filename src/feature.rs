use std::marker::PhantomData;
use std::ops::AddAssign;

use derivative::*;
use geo::CoordFloat;
use geo::Coordinate;
use geo::Geometry;
use geo::LineString;
use geo::MultiLineString;
use geo::MultiPoint;
use geo::MultiPolygon;
use geo::Point;
use geo::Polygon;
use topojson::Arc;
use topojson::ArcIndexes;
use topojson::NamedGeometry;
use topojson::Topology;
use topojson::Value;

use crate::reverse::reverse;
use crate::transform::generate as transform;
use crate::transform::TransformFn;

#[derive(Derivative)]
#[derivative(Debug)]
struct Builder<T>
where
    T: AddAssign<T> + CoordFloat,
{
    pd: PhantomData<T>,
    arcs: Vec<Arc>,
    #[derivative(Debug = "ignore")]
    transform_point: TransformFn<T>,
}

// impl From<(Topology, String)> for Object {
//     #[inline]
//     fn from(tuple: (Topology, String)) -> Self {
//         Object::feature(tuple.0, tuple.0[tuple.1]).expect("failed to parse")
//     }
// }

impl<T> Builder<T>
where
    T: AddAssign<T> + CoordFloat,
{
    /// A constructor that fails when topology does not contain a transform.
    #[inline]
    pub fn generate(topology: Topology, o: NamedGeometry) -> Option<Geometry<T>>
    where
        T: 'static + CoordFloat,
    {
        match topology.transform {
            None => None,
            Some(transform_params) => {
                let transform = transform::<T>(transform_params);
                let mut out = Self {
                    pd: PhantomData::<T>::default(),
                    arcs: topology.arcs,
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
        // let n = self.arcs.len();
        for (k, v) in self.arcs[index].iter().enumerate() {
            let t = (self.transform_point)(v, k);
            points.push((t[0], t[1]));
        }

        if i < 0 {
            reverse(points, self.arcs.len());
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
        let mut points: Vec<(T, T)> = Vec::new();
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
    fn geometry(&mut self, o: NamedGeometry) -> Geometry<T> {
        match &o.geometry.value {
            Value::GeometryCollection(_stopo_geometries) => {
                todo!("Must implement GeometryCollection");
                // let geo_geometries: Vec<Geometry<T>>;
                // for topo_geometry in topo_geometries {
                //     let topo_geometry = topo_geometry.value;
                //     let geo_geometry = Geomtry<T>::parse().unwrap();
                //     geo_geometries.push(geo_geometry);
                // }
                // Geometry::GeometryCollection(GeometryCollection(geo_geometries))
            }
            Value::Point(topo_point) => {
                // Should I transform using self.point()??
                let p = self.point(topo_point);
                Geometry::Point(Point(Coordinate::<T> { x: p[0], y: p[1] }))
            }
            Value::MultiPoint(topo_multipoint) => {
                let coordinates: Vec<Coordinate<T>> = topo_multipoint
                    .iter()
                    .map(|c| Coordinate {
                        x: T::from(c[0]).unwrap(),
                        y: T::from(c[1]).unwrap(),
                    })
                    .collect();
                let geo_multipoint: MultiPoint<T> = coordinates.into();
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
    use geo::Polygon;
    use pretty_assertions::assert_eq;
    use topojson::NamedGeometry;
    use topojson::TransformParams;
    use topojson::Value;

    #[test]
    fn geometry_type_is_preserved() {
        println!("topojson.feature the geometry type is preserved");
        let t = simple_topology(topojson::Geometry::new(Value::Polygon(vec![vec![0]])));
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "a".into(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![0]])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "a".into(),
                geometry: topojson::Geometry::new(Value::Point(vec![0_f64, 0_f64])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::MultiPoint(vec![vec![0_f64, 0_f64]])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::LineString(vec![0])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::MultiLineString(vec![vec![0]])),
            },
        );

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
        let computed1: Option<Geometry<f64>> = Builder::<f64>::generate(
            t1,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::LineString(vec![0])),
            },
        );

        assert_eq!(
            computed1,
            Some(Geometry::LineString(LineString(vec![
                Coordinate { x: 0_f64, y: 0_f64 },
                Coordinate { x: 1_f64, y: 0_f64 },
                Coordinate { x: 1_f64, y: 1_f64 },
                Coordinate { x: 0_f64, y: 1_f64 },
                Coordinate { x: 0_f64, y: 0_f64 },
            ])))
        );

        let t2 = simple_topology(topojson::Geometry::new(Value::MultiLineString(vec![vec![
            3,
        ]])));
        let computed2: Option<Geometry<f64>> = Builder::<f64>::generate(
            t2,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::MultiLineString(vec![vec![3], vec![4]])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::Polygon(vec![vec![0]])),
            },
        );

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
        let computed: Option<Geometry<f64>> = Builder::<f64>::generate(
            t,
            NamedGeometry {
                name: "foo".into(),
                geometry: topojson::Geometry::new(Value::MultiPolygon(vec![vec![vec![0]]])),
            },
        );

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
