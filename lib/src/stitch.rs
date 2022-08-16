use std::collections::HashMap;

use topojson::{ArcIndexes, Topology};

use crate::translate;

pub(super) fn stitch(topology: &Topology, mut arcs: ArcIndexes) -> Vec<ArcIndexes> {
    let mut stitch = Stitch {
        stitched_arcs: vec![],
        fragment_by_start: HashMap::new(),
        fragment_by_end: HashMap::new(),
        fragments: vec![],
        // Special case, JS uses, -1 which is not availble here.
        empty_index: usize::max_value(),
        topology,
    };

    // Stitch empty arcs first, since they may be subsumed by other arcs.
    // Cannot use conventional iterator here as we are swapping
    // element as we loop.
    for j in 0..arcs.len() {
        let i = arcs[j];
        let arc = &mut topology.arcs[translate(i)].clone();
        if arc.len() < 3usize && arc[1][0] == 0_f64 && arc[1][1] == 0_f64 {
            stitch.empty_index += 1;
            arcs.swap(stitch.empty_index, j);
        }
    }

    stitch.flush(
        &mut stitch.fragment_by_end.clone(),
        &mut stitch.fragment_by_start.clone(),
    );
    stitch.flush(
        &mut stitch.fragment_by_start.clone(),
        &mut stitch.fragment_by_end.clone(),
    );

    stitch.fragments
}

#[derive(Clone, Debug)]
struct Fragment {
    items: Vec<i32>,
    start: Option<[i32; 2]>,
    end: Option<[i32; 2]>,
}

#[derive(Clone, Debug)]
struct Stitch<'a> {
    stitched_arcs: ArcIndexes,
    fragment_by_start: HashMap<String, Fragment>,
    fragment_by_end: HashMap<String, Fragment>,
    fragments: Vec<ArcIndexes>,
    empty_index: usize,
    topology: &'a Topology,
}

impl<'a> Stitch<'a> {
    // Stitch empty arcs first, since they may be subsumed by other arcs.
    fn ends(&self, i: i32) -> Vec<Vec<f64>> {
        let arc = &self.topology.arcs[translate(i)];
        let p0 = arc[0].clone();
        let mut p1: Vec<f64>;

        if self.topology.transform.is_some() {
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
    }

    fn flush(
        &mut self,
        fragment_by_end: &mut HashMap<String, Fragment>,
        fragment_by_start: &mut HashMap<String, Fragment>,
    ) {
        for k in fragment_by_end.keys() {
            fragment_by_start.remove(k);
            if let Some(f) = fragment_by_end.clone().get_mut(k) {
                f.start = None;
                f.end = None;

                for i in f.items.iter() {
                    self.stitched_arcs[translate(*i)] = 1_i32;
                }

                self.fragments.push(f.items.clone())
            }
        }
    }
}
