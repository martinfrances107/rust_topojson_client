use derivative::Derivative;

use geo::line_string;
use geo::CoordFloat;
use geo::Geometry;
use geo::GeometryCollection;
use geo::LineString;
use geo::MultiLineString;
use geo::MultiPoint;
use geo::MultiPolygon;
use geo::Point;
use geo::Polygon;
use geo_types::Coord;
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
pub fn feature_from_name<T>(topology: &Topology, name: &str) -> Option<Geometry<T>>
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

#[derive(Derivative)]
#[derivative(Debug)]
/// State holds data extracted from a Topological object.
struct Builder {
    arcs: Vec<Arc>,
    #[derivative(Debug = "ignore")]
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
                let geo_geometries: Vec<Geometry<T>> = topo_geometries
                    .iter()
                    .map(|x| self.geometry(&x.value))
                    .collect();
                Geometry::GeometryCollection(GeometryCollection(geo_geometries))
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
                let geo_multipoint = MultiPoint(points);
                Geometry::MultiPoint(geo_multipoint)
            }
            Value::LineString(topo_ls) => {
                let line = self.line(topo_ls);
                let geo_ls: LineString<T> = line
                    .iter()
                    .map(|p| Coord {
                        x: T::from(p.0).unwrap(),
                        y: T::from(p.1).unwrap(),
                    })
                    .collect();
                Geometry::LineString(geo_ls)
            }

            Value::MultiLineString(topo_mls) => {
                let geo_mls: Vec<LineString<T>> = topo_mls
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
                Geometry::MultiLineString(MultiLineString(geo_mls))
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
                    None => Geometry::Polygon(Polygon::new(line_string![], vec![])),
                }
            }
            Value::MultiPolygon(topo_mp) => {
                let geo_polygon: Vec<Polygon<T>> = topo_mp
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

                Geometry::MultiPolygon(MultiPolygon(geo_polygon))
            }
        }
    }
}
