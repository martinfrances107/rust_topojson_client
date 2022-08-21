use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::num::Wrapping;

use topojson::{ArcIndexes, Topology};

use crate::translate;

pub(super) fn stitch(topology: &Topology, mut arcs: ArcIndexes) -> Vec<ArcIndexes> {
    let mut stitch = Stitch {
        stitched_arcs: HashSet::new(),
        fragment_by_start: BTreeMap::new(),
        fragment_by_end: BTreeMap::new(),
        fragments: vec![],
        topology,
    };

    // In javascript emptyIndex = -1
    // In RUST that is, what when incremented will be 0usize.
    let mut empty_index = Wrapping(usize::MAX);

    // Stitch empty arcs first, since they may be subsumed by other arcs.
    // Cannot use conventional iterator here as we are swapping
    // element as we loop.
    for j in 0..arcs.len() {
        let i = arcs[j];
        let arc = &topology.arcs[translate(i)];
        if arc.len() < 3usize && arc[1][0] == 0_f64 && arc[1][1] == 0_f64 {
            empty_index += 1;
            arcs.swap(empty_index.0, j);
        }
    }

    arcs.iter().for_each(|i| {
        let e = stitch.ends(*i);
        // TODO could I use  or_default() instead of .unwrap()
        let start: FragmentKey = gen_key(e.get(0).unwrap());
        let end = gen_key(e.get(1).unwrap());

        let f = stitch.fragment_by_end.get(&start);
        if f.is_some() {
            // let key = *f.clone().unwrap().end.as_ref().unwrap().clone().to_string();
            let key = f.unwrap().end.as_ref().unwrap().clone();
            stitch.fragment_by_end.remove(&key);

            // Need to recheck for the exitence of f, as it may have just been .removed().
            // We can't like Javascript push undefined at this point.
            if let Some(f) = stitch.fragment_by_end.get(&start) {
                let mut f = f.to_owned();
                f.items.push_back(*i);
                f.end = Some(end.clone());

                if let Some(g) = stitch.fragment_by_start.get(&end) {
                    let g = g.clone();
                    stitch.fragment_by_start.remove(&start);

                    let fg = if g == f {
                        f.clone()
                    } else {
                        let f_then_g = f.items.into_iter().chain(g.items.into_iter());
                        Fragment {
                            items: VecDeque::from_iter(f_then_g),
                            start: f.start.clone(),
                            end: g.end,
                        }
                    };

                    stitch
                        .fragment_by_start
                        .insert(fg.start.clone().unwrap(), fg.clone());
                    stitch.fragment_by_end.insert(fg.end.clone().unwrap(), fg);
                } else {
                    stitch
                        .fragment_by_start
                        .insert(f.start.clone().unwrap(), f.clone());
                    stitch.fragment_by_end.insert(f.end.clone().unwrap(), f);
                }
            }
        } else if let Some(f) = stitch.fragment_by_start.get(&end) {
            let mut f = f.to_owned();
            stitch.fragment_by_start.remove(&f.start.unwrap());
            f.items.push_front(*i);
            f.start = Some(start.clone());

            if let Some(g) = stitch.fragment_by_end.get(&start) {
                let g = g.clone();
                stitch.fragment_by_end.remove(g.end.as_ref().unwrap());
                let gf = if g == f {
                    f.clone()
                } else {
                    let g_then_f = g
                        .items
                        .clone()
                        .into_iter()
                        .chain(f.items.clone().into_iter());
                    Fragment {
                        items: VecDeque::from_iter(g_then_f),
                        start: g.start.clone(),
                        end: f.end.clone(),
                    }
                };

                stitch
                    .fragment_by_start
                    .insert(gf.start.clone().unwrap(), gf.clone());
                stitch.fragment_by_end.insert(gf.clone().end.unwrap(), gf);
            } else {
                stitch
                    .fragment_by_start
                    .insert(f.start.clone().unwrap(), f.clone());
                stitch.fragment_by_end.insert(f.end.clone().unwrap(), f);
            }
        } else {
            let f = Fragment {
                items: VecDeque::from(vec![*i]),
                start: Some(start.clone()),
                end: Some(end.clone()),
            };
            stitch.fragment_by_start.insert(start, f.clone());
            stitch.fragment_by_end.insert(end, f);
        }
    });

    stitch.flush(
        &mut stitch.fragment_by_end.clone(),
        &mut stitch.fragment_by_start.clone(),
    );
    stitch.flush(
        &mut stitch.fragment_by_start.clone(),
        &mut stitch.fragment_by_end.clone(),
    );

    // Conversion of Vec<Fragment> to Vec<ArcIndexes>
    //
    // Compared with JS there is an extra loop here
    // which has time and memory implications.
    let mut fragments_plain: Vec<ArcIndexes> = stitch
        .fragments
        .iter()
        .map(|f| Vec::from(f.items.clone()))
        .collect();

    arcs.iter().for_each(|i| {
        if stitch.stitched_arcs.contains(&translate(*i)) {
            fragments_plain.push(vec![*i]);
        }
    });
    fragments_plain
}

// Returns a key, used in the Fragment struct.
fn gen_key(input: &[f64]) -> FragmentKey {
    let output_str: Vec<String> = input
        .iter()
        .map(|f| {
            let int = *f as i32;
            int.to_string()
        })
        .collect();
    output_str.join(",")
}

#[derive(Clone, Debug, PartialEq)]
struct Fragment {
    pub items: VecDeque<i32>,
    start: Option<String>,
    end: Option<String>,
}

type FragmentKey = String;

#[derive(Clone, Debug)]
struct Stitch<'a> {
    stitched_arcs: HashSet<usize>,
    fragment_by_start: BTreeMap<FragmentKey, Fragment>,
    fragment_by_end: BTreeMap<FragmentKey, Fragment>,
    fragments: Vec<Fragment>,
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

    /// Iterate over fragment_by_end :-
    /// deleting elements in fragment_by_start
    /// building stitched_by_arcs and fragments.
    fn flush(
        &mut self,
        fragment_by_end: &mut BTreeMap<FragmentKey, Fragment>,
        fragment_by_start: &mut BTreeMap<FragmentKey, Fragment>,
    ) {
        for (k, f) in fragment_by_end.iter() {
            fragment_by_start.remove(k);
            let mut f = f.clone();
            f.start = None;
            f.end = None;
            for i in f.items.iter() {
                self.stitched_arcs.insert(translate(*i));
            }
            self.fragments.push(f)
        }
    }
}
