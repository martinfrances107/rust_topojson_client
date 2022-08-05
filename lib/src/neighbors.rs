use topojson::{ ArcIndexes,  NamedGeometry,  Value};

use crate::bisect::bisect;

#[derive(Debug)]
struct Neighbors {
    indexesByArc: ArcIndexes,
	neighbors: Vec<i32>,
	o: Vec<NamedGeometry>,
}

impl Neighbors {
    // fn new(objects: &mut [NamedGeometry]) -> Vec<Neighbors> {
    //     let indexesByArc: Vec<i32> = vec![];
    //     let neightbors: Vec<Vec<i32>> = objects
    //         .iter()
    //         .map(|_| {
    //             return vec![];
    //         })
    //         .collect();

    //     for o in objects {
    //         Neighbors::geometry(o, 0);
    //     }

    //     for i in indexesByArc {
    //       let indexes = indexesByArc[i as usize];
    //       let m = indexes.len();
    //       for j in 0..m{
    //         for k in j+1..m{
    //           let ij = indexes[i as usize];
    //           let ik = indexes[k as usize];
    //           let n = neightbors[ij];
    //           i = bisect(n, ik);

    //           if n[i] != ik {
    //             n.insert(i, ik);
    //           }

    //           let n = neightbors[ik];
    //           let i = bisect(n, ij);
    //           if n != ij {
    //             n.insert(i,0, ij);
    //           }
    //         }
    //       }
    //       for indexes in indexesByArc[i] {}
	// 	  vec![]
    //     }
    // }

    // fn line(&mut self, arcs: &mut ArcIndexes, i: usize) {
    //     for a in arcs {
    //         if *a < 0 {
    //             *a = !*a;
    //         };

    //         match self.indexesByArc.get(*a as usize ) {
    //             Some(o) => {
    //                 self.indexesByArc[*a as usize].push(i);
    //             }
    //             None => self.indexesByArc[*a as usize] = vec![i],
    //         }
    //     }
    // }

    // fn polygons(self, arcs: &mut ArcIndexes, i: i32) {
    //     for arc in arcs {
    //         self.line(arc, i);
    //     }
    // }

    // fn geometry(o: &mut NamedGeometry, i: i32) {
    //     match o.geometry.value {
    //         Value::GeometryCollection(gc) => {}
    //         Value::LineString(line) => {}
    //         Value::MultiLineString(polygon) => {}
    //         Value::Polygon(polygon) => {}
    //         Value::MultiPolygon(mp) => {
    //             // function (arcs, i) { arcs.forEach(function (arc) { polygon(arc, i); }); }
    //         }
    //         _ => {
    //             todo!("What is mising here!");
    //         }
    //     }
    // }
}

// #[cfg(not(tarpaulin_include))]
// #[cfg(test)]
// mod neighbors_tests {

//     use super::*;
//     use pretty_assertions::assert_eq;
//     use topojson::Geometry;
//     use topojson::NamedGeometry;
//     use topojson::Topology;
//     use topojson::TransformParams;
//     use topojson::Value;

//     #[test]
//     fn empty_array_empty_input() {
//         println!("neighbors returns an empty array for empty input");
//         assert_eq!(Neighbors::new(&mut []).len(),0);
//     }

//       //
//       // A-----B
//       //
//       // C-----D
//       //
//     #[test]
//     fn empty_array_for_objects_with_no_neighbors() {
//         println!("neighbors returns an empty array for objects with no neighbors");

//         let topology = Topology {
//             arcs: vec![
//                 vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
//                 vec![vec![0_f64, 1_f64], vec![1_f64, 1_f64]],
//             ],

//             objects: vec![
//                 NamedGeometry {
//                     name: "ab".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![0])),
//                 },
//                 NamedGeometry {
//                     name: "cd".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![1])),
//                 },
//             ],
//             bbox: Some(vec![1_f64, 2_f64, 3_f64, 4_f64]),
//             transform: None,
//             foreign_members: None,
//         };

//         let n = Neighbors::new(&mut topology.o);
//         assert_eq!(n, vec![vec![], vec![]]);
//     }

//     //
//     // A-----B-----C
//     //
//     #[test]
//     fn only_share_isolated_points_are_not_considered_neighbors() {
//         println!(
//             "neighbors geometries that only share isolated points are not considered neighbors"
//         );
//         let topology = Topology {
//             objects: vec![
//                 NamedGeometry {
//                     name: "ab".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![0])),
//                 },
//                 NamedGeometry {
//                     name: "bc".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![1])),
//                 },
//             ],
//             arcs: vec![
//                 vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
//                 vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
//             ],
//             bbox: None,
//             transform: None,
//             foreign_members: None,
//         };

//         assert_eq!(Neighbors::new(&mut topology.objects), vec![vec![], vec![]]);
//     }

//     //
//     // A-----B-----C-----D
//     //
//     #[test]
//     fn neighbors_geometries_that_share_arcs_are_considered_neighbors() {
//         println!("neighbors geometries that share arcs are considered neighbors");
//         let topology = Topology {
//             objects: vec![
//                 NamedGeometry {
//                     name: "ab".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![0])),
//                 },
//                 NamedGeometry {
//                     name: "bc".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![1])),
//                 },
//             ],
//             arcs: vec![
//                 vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
//                 vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
//                 vec![vec![2_f64, 0_f64], vec![3_f64, 0_f64]],
//             ],
//             bbox: None,
//             transform: None,
//             foreign_members: None,
//         };

//         assert_eq!(Neighbors::new(&mut topology.objects), vec![vec![1], vec![0]]);
//     }

//     //
//     // A-----B-----C-----D-----E-----F
//     //
//     #[test]
//     fn neighbors_are_returned_in_sorted_order_by_index() {
//         println!("neighbors neighbors are returned in sorted order by index");
//         let topology = Topology {
//             objects: vec![
//                 NamedGeometry {
//                     name: "abcd".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![0, 1, 2])),
//                 },
//                 NamedGeometry {
//                     name: "bcde".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![1, 2, 3])),
//                 },
//                 NamedGeometry {
//                     name: "cdef".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![2, 3, 4])),
//                 },
//                 NamedGeometry {
//                     name: "dbca".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![-3, -2, -1])),
//                 },
//                 NamedGeometry {
//                     name: "edcb".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![-4, -3, -2])),
//                 },
//                 NamedGeometry {
//                     name: "fedc".to_string(),
//                     geometry: Geometry::new(Value::LineString(vec![-5, -4, -3])),
//                 },
//             ],
//             arcs: vec![
//                 vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
//                 vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
//                 vec![vec![2_f64, 0_f64], vec![3_f64, 0_f64]],
//                 vec![vec![3_f64, 0_f64], vec![4_f64, 0_f64]],
//                 vec![vec![4_f64, 0_f64], vec![5_f64, 0_f64]],
//             ],
//             bbox: None,
//             transform: None,
//             foreign_members: None,
//         };

//         assert_eq!(Neighbors::new(&mut topology.objects), vec![vec![1], vec![0]]);
//     }
// }
