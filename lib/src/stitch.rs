use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::num::Wrapping;
use std::rc::Rc;

use topojson::{ArcIndexes, Topology};

use crate::translate;

pub(super) fn stitch(
    topology: &Topology,
    mut arcs: ArcIndexes,
) -> Vec<ArcIndexes> {
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
    // Cannot use conventional iterator here as we are swapping element as
    // we loop.
    for j in 0..arcs.len() {
        let i = arcs[j];
        let arc = &topology.arcs[translate(i)];
        if arc.len() < 3usize && arc[1][0] == 0_f64 && arc[1][1] == 0_f64 {
            empty_index += 1;
            arcs.swap(empty_index.0, j);
        }
    }

    for i in &arcs {
        let e = stitch.ends(*i);
        // TODO could I use  or_default() instead of .unwrap()
        let start: FragmentKey = gen_key(e.first().unwrap());
        let end = gen_key(e.get(1).unwrap());

        if let Some(f) = stitch.fragment_by_end.clone().get(&start) {
            let key = *f.clone().borrow_mut().end.as_ref().unwrap();
            stitch.fragment_by_end.remove(&key);
            f.borrow_mut().items.push_back(*i);
            f.borrow_mut().end = Some(end);

            if let Some(g) = stitch.fragment_by_start.get(&end) {
                let g = g.clone();
                stitch
                    .fragment_by_start
                    .remove(&(g).borrow_mut().start.as_ref().unwrap().clone());

                let fg = if g == *f {
                    f.clone()
                } else {
                    let g_items = g.borrow().items.clone();
                    let items = f
                        .borrow_mut()
                        .items
                        .iter()
                        .chain(g_items.iter())
                        .copied()
                        .collect();
                    let g_end = (g).borrow().end.unwrap();
                    Rc::new(RefCell::new(Fragment {
                        items,
                        start: f.borrow_mut().start,
                        end: Some(g_end),
                    }))
                };
                let key = *f.clone().borrow_mut().end.as_ref().unwrap();
                stitch.fragment_by_start.insert(key, fg.clone());
                let key = fg.borrow_mut().end.unwrap();
                stitch.fragment_by_end.insert(key, fg);
            } else {
                stitch
                    .fragment_by_start
                    .insert(f.borrow_mut().start.unwrap(), f.clone());
                stitch
                    .fragment_by_end
                    .insert(f.borrow_mut().end.unwrap(), f.clone());
            }
        } else if let Some(f) = stitch.fragment_by_start.get(&end) {
            let f = f.clone();
            let key = *f.borrow_mut().start.as_ref().unwrap();
            stitch.fragment_by_start.remove(&key);
            f.borrow_mut().items.push_front(*i);
            f.borrow_mut().start = Some(start);

            if let Some(g) = stitch.fragment_by_end.get(&start) {
                let g = g.clone();
                stitch
                    .fragment_by_end
                    .remove(g.borrow().end.as_ref().unwrap());

                let gf = if g == f {
                    f
                } else {
                    let g_then_f = g
                        .borrow()
                        .items
                        .clone()
                        .into_iter()
                        .chain(f.borrow_mut().items.clone());

                    Rc::new(RefCell::new(Fragment {
                        items: g_then_f.collect::<VecDeque<_>>(),
                        start: g.borrow().start,
                        end: f.borrow_mut().end,
                    }))
                };

                let key = gf.borrow().start.unwrap();
                stitch.fragment_by_start.insert(key, gf.clone());
                let key = gf.borrow().end.unwrap();
                stitch.fragment_by_end.insert(key, gf);
            } else {
                let key = f.borrow_mut().start.unwrap();
                stitch.fragment_by_start.insert(key, f.clone());
                let key = f.borrow_mut().end.unwrap();
                stitch.fragment_by_end.insert(key, f);
            }
        } else {
            let f = Rc::new(RefCell::new(Fragment {
                items: VecDeque::from(vec![*i]),
                start: Some(start),
                end: Some(end),
            }));
            stitch.fragment_by_start.insert(start, f.clone());
            stitch.fragment_by_end.insert(end, f);
        }
    }

    stitch.flush(&FlushDir::EndToStart);
    stitch.flush(&FlushDir::StartToEnd);

    // Conversion of Vec<Fragment> to Vec<ArcIndexes>
    //
    // Compared with JS there is an extra loop here which has time and
    // memory implications.
    let mut fragments_plain: Vec<ArcIndexes> = stitch
        .fragments
        .iter()
        .map(|f| Vec::from(f.items.clone()))
        .collect();

    for i in &arcs {
        if !stitch.stitched_arcs.contains(&translate(*i)) {
            fragments_plain.push(vec![*i]);
        }
    }

    fragments_plain
}

// Returns a key, used in the Fragment struct.
fn gen_key(input: &[f64]) -> FragmentKey {
    debug_assert_eq!(input.len(), 2);
    (input[0] as i32, input[1] as i32)
}

#[derive(Clone, Debug, PartialEq)]
struct Fragment {
    pub items: VecDeque<i32>,
    start: Option<FragmentKey>,
    end: Option<FragmentKey>,
}

type FragmentKey = (i32, i32);

#[derive(Clone, Debug)]
struct Stitch<'a> {
    stitched_arcs: HashSet<usize>,
    fragment_by_start: BTreeMap<FragmentKey, Rc<RefCell<Fragment>>>,
    fragment_by_end: BTreeMap<FragmentKey, Rc<RefCell<Fragment>>>,
    fragments: Vec<Fragment>,
    topology: &'a Topology,
}

enum FlushDir {
    EndToStart,
    StartToEnd,
}

impl Stitch<'_> {
    // Stitch empty arcs first, since they may be subsumed by other arcs.
    fn ends(&self, i: i32) -> Vec<Vec<f64>> {
        let arc = &self.topology.arcs[translate(i)];
        let p0 = arc[0].clone();
        let mut p1: Vec<f64>;

        if self.topology.transform.is_some() {
            p1 = vec![0_f64, 0_f64];
            for dp in arc {
                p1[0] += dp[0];
                p1[1] += dp[1];
            }
        } else {
            p1 = arc.last().unwrap().clone();
        }
        if i < 0 {
            vec![p1, p0]
        } else {
            vec![p0, p1]
        }
    }

    /// Iterate over `fragment_by_end` :-
    /// deleting elements in `fragment_by_start`
    /// building `stitched_by_arcs` and fragments.
    fn flush(&mut self, direction: &FlushDir) {
        let (fragment_by_end, fragment_by_start) = match direction {
            FlushDir::StartToEnd => {
                (&mut self.fragment_by_start, &mut self.fragment_by_end)
            }
            FlushDir::EndToStart => {
                (&mut self.fragment_by_end, &mut self.fragment_by_start)
            }
        };

        let search_iterator =
            fragment_by_end.keys().copied().collect::<Vec<(i32, i32)>>();
        for k in search_iterator {
            let f = fragment_by_end.get(&k).unwrap().clone();

            fragment_by_start.remove(&k);
            // fragment_by_end.remove(&cross_link);

            let mut f = f.borrow_mut();
            f.start = None;
            f.end = None;

            for i in &f.items {
                self.stitched_arcs.insert(translate(*i));
            }
            self.fragments.push(f.clone());
        }
    }
}
