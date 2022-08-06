use std::{cell::RefCell,  rc::Rc};
use topojson::{ArcIndexes,  NamedGeometry, Value};

use crate::bisect::bisect;

fn neighbors(objects: &mut [NamedGeometry]) -> Vec<Vec<i32>> {
    let indexes_by_arc: Rc<RefCell<Vec<ArcIndexes>>> = Rc::new(RefCell::new(vec![vec![]]));
    let mut neighbors: Vec<Vec<i32>> = objects.iter().map(|_| vec![]).collect();

    let line = |arcs: &mut ArcIndexes, i: i32| {
        for a in arcs {
            let index: usize = if *a < 0 { !*a as usize } else { *a as usize };

            match indexes_by_arc.borrow_mut().get_mut(index) {
                Some(o) => o.push(i),
                None => indexes_by_arc.borrow_mut()[index] = vec![i],
            }
        }
    };

    let polygon = |arcs: &mut Vec<ArcIndexes>, i: i32| {
        for mut arc in arcs {
            line(arc, i);
        }
    };

    let named_geometry = |o: &mut NamedGeometry, i: i32| {
        match &mut o.geometry.value {
            Value::GeometryCollection(gc) => {
                for elem in gc {
                    todo!();
                }
            }
            Value::LineString(l) => line(l, i),
            Value::MultiLineString(p) => polygon(p, i),
            Value::Polygon(p) => polygon(p, i),
            Value::MultiPolygon(_mp) => {
                todo!()
                // function (arcs, i) { arcs.forEach(function (arc) { polygon(arc, i); }); }
            }
            _ => {
                todo!("What is mising here!");
            }
        }
    };

    for o in objects {
        named_geometry( o, 0);
    }

    for (i, indexes_i) in indexes_by_arc.borrow().iter().enumerate() {
        let m = indexes_i.len();
        for j in 0..m {
            for k in j + 1..m {
                let ij = indexes_i[i] as usize;
                let ik = indexes_i[k];
                let n = &mut neighbors[ij];
                let b = bisect(n, ik as i32);

                if n[b] != ik as i32 {
                    n.insert(b, ik as i32);
                }

                let n = &mut neighbors[ik as usize];
                let b = bisect(n, ij as i32);
                if n[b] != ij as i32 {
                    n.insert(b, ij as i32);
                }
            }
        }
    }
    neighbors
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod neighbors_tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_array_empty_input() {
        println!("neighbors returns an empty array for empty input");
        assert_eq!(neighbors(&mut []).len(), 0);
    }

    // //
    // // A-----B
    // //
    // // C-----D
    // //
    // #[test]
    // fn empty_array_for_objects_with_no_neighbors() {
    //     println!("neighbors returns an empty array for objects with no neighbors");

    //     let mut topology = Topology {
    //         arcs: vec![
    //             vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
    //             vec![vec![0_f64, 1_f64], vec![1_f64, 1_f64]],
    //         ],

    //         objects: vec![
    //             NamedGeometry {
    //                 name: "ab".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![0])),
    //             },
    //             NamedGeometry {
    //                 name: "cd".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![1])),
    //             },
    //         ],
    //         bbox: Some(vec![1_f64, 2_f64, 3_f64, 4_f64]),
    //         transform: None,
    //         foreign_members: None,
    //     };

    //     let n = neighbors(&mut topology.objects);
    //     let expected: Vec<Vec<i32>> =vec![vec![], vec![]];
    //     assert_eq!(n, expected);
    // }

    // //
    // // A-----B-----C
    // //
    // #[test]
    // fn only_share_isolated_points_are_not_considered_neighbors() {
    //     println!(
    //         "neighbors geometries that only share isolated points are not considered neighbors"
    //     );
    //     let mut topology = Topology {
    //         objects: vec![
    //             NamedGeometry {
    //                 name: "ab".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![0])),
    //             },
    //             NamedGeometry {
    //                 name: "bc".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![1])),
    //             },
    //         ],
    //         arcs: vec![
    //             vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
    //             vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
    //         ],
    //         bbox: None,
    //         transform: None,
    //         foreign_members: None,
    //     };

    //     let expected: Vec<Vec<i32>> =vec![vec![1], vec![0]];
    //     assert_eq!(neighbors(&mut topology.objects), expected);
    // }

    // //
    // // A-----B-----C-----D
    // //
    // #[test]
    // fn neighbors_geometries_that_share_arcs_are_considered_neighbors() {
    //     println!("neighbors geometries that share arcs are considered neighbors");
    //     let mut topology = Topology {
    //         objects: vec![
    //             NamedGeometry {
    //                 name: "ab".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![0])),
    //             },
    //             NamedGeometry {
    //                 name: "bc".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![1])),
    //             },
    //         ],
    //         arcs: vec![
    //             vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
    //             vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
    //             vec![vec![2_f64, 0_f64], vec![3_f64, 0_f64]],
    //         ],
    //         bbox: None,
    //         transform: None,
    //         foreign_members: None,
    //     };

    //     let expected: Vec<Vec<i32>> =vec![vec![1], vec![0]];
    //     assert_eq!(neighbors(&mut topology.objects), expected);
    // }

    // //
    // // A-----B-----C-----D-----E-----F
    // //
    // #[test]
    // fn neighbors_are_returned_in_sorted_order_by_index() {
    //     println!("neighbors neighbors are returned in sorted order by index");
    //     let mut topology = Topology {
    //         objects: vec![
    //             NamedGeometry {
    //                 name: "abcd".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![0, 1, 2])),
    //             },
    //             NamedGeometry {
    //                 name: "bcde".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![1, 2, 3])),
    //             },
    //             NamedGeometry {
    //                 name: "cdef".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![2, 3, 4])),
    //             },
    //             NamedGeometry {
    //                 name: "dbca".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![-3, -2, -1])),
    //             },
    //             NamedGeometry {
    //                 name: "edcb".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![-4, -3, -2])),
    //             },
    //             NamedGeometry {
    //                 name: "fedc".to_string(),
    //                 geometry: Geometry::new(Value::LineString(vec![-5, -4, -3])),
    //             },
    //         ],
    //         arcs: vec![
    //             vec![vec![0_f64, 0_f64], vec![1_f64, 0_f64]],
    //             vec![vec![1_f64, 0_f64], vec![2_f64, 0_f64]],
    //             vec![vec![2_f64, 0_f64], vec![3_f64, 0_f64]],
    //             vec![vec![3_f64, 0_f64], vec![4_f64, 0_f64]],
    //             vec![vec![4_f64, 0_f64], vec![5_f64, 0_f64]],
    //         ],
    //         bbox: None,
    //         transform: None,
    //         foreign_members: None,
    //     };

    //     let expected: Vec<Vec<i32>> =vec![vec![], vec![]];
    //     assert_eq!(neighbors(&mut topology.objects), expected);
    // }
}
