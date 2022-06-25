use std::{collections::HashMap, marker::PhantomData};

use geo::CoordFloat;
use topojson::{ArcIndexes, Topology};

struct Fragment {
    b: bool,
    start: [i32; 2],
    end: [i32; 2],
}
pub struct Stitch {
    stitched_arcs: Vec<ArcIndexes>,
    // fragment_by_start: Vec<[f64; 2]>,
    // fragment_by_end: Vec<[f64; 2]>,
    // fragments: Vec<[f64; 2]>,
    fragment_by_start: HashMap<String, Fragment>,
    fragment_by_end: HashMap<String, Fragment>,
    fragments: Vec<Fragment>,
    empty_index: usize,
}

impl Default for Stitch {
    fn default() -> Self {
        Stitch {
            stitched_arcs: vec![],
            fragment_by_start: HashMap::new(),
            fragment_by_end: HashMap::new(),
            fragments: vec![],
            // Special case, JS uses, -1 which is not availble here.
            empty_index: usize::max_value(),
        }
    }
}

impl Stitch {
    pub fn gen(&mut self, topology: Topology, mut arcs: ArcIndexes) {
        let stitch = Stitch::default();

        let ends = |i: i32| -> Vec<Vec<f64>> {
            let index = if i < 0 { !i as usize } else { i as usize };

            let arc = &topology.arcs[index];
            let p0 = arc[0].clone();
            let mut p1: Vec<f64>;

            if topology.transform.is_some() {
                p1 = vec![0_f64, 0_f64];
                arc.iter().for_each(|dp| {
                    p1[0] += dp[0];
                    p1[1] += dp[1];
                });
            } else {
                p1 = arc.last().unwrap().clone();
            }
            if i < 0 {
                vec![p1, p0]
            } else {
                vec![p0, p1]
            }
        };

        let flush = |fragment_by_end: HashMap<String, Fragment>,
                     fragment_by_start: HashMap<String, Fragment>| {
            // for k in fragment_by_end {
            //     let f = k.get(k.0);
            //     fragment_by_start.remove(f.start);
            //     f.start.delete();
            //     f.end.delete();

            //     f.map(|i| {
            //         i = if i < 0 { !i } else { i };
            //         stitch.stitched_arcs[i];
            //     });
            //     self.fragments.push(f);
            // }
        };

        arcs.iter_mut().enumerate().map(|(j, i)| {
            // let index = if *i < 0 { !*i as usize } else { *i as usize };
            // let arc = &mut topology.arcs[index].clone();
            // if arc.len() < 3usize && arc[1][0] == 0_f64 && arc[1][1] == 0_f64 {
            //     self.empty_index += 1;
            //     let t = arcs[stitch.empty_index];
            //     arcs[stitch.empty_index] = *i;
            //     arcs[j] = t;
            // }
        });

        // arcs.map(|i| {})
    }
}
