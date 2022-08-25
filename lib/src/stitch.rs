use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::num::Wrapping;
use std::rc::Rc;

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

        if let Some(f) = stitch.fragment_by_end.clone().get(&start) {
            let key = f
                .clone()
                .fragment
                .borrow_mut()
                .end
                .as_ref()
                .unwrap()
                .clone();
            stitch.fragment_by_end.remove(&key);
            f.fragment.borrow_mut().items.push_back(*i);
            f.fragment.borrow_mut().end = Some(end);

            if let Some(g) = stitch.fragment_by_start.get(&end) {
                let g = g.clone();
                stitch
                    .fragment_by_start
                    .remove(&(g).fragment.borrow_mut().start.as_ref().unwrap().clone());

                let mut fg = if g == *f {
                    f.clone()
                } else {
                    let g_items = g.fragment.borrow().items.clone();
                    let items = f
                        .fragment
                        .borrow_mut()
                        .items
                        .iter()
                        .chain(g_items.iter())
                        .copied()
                        .collect();
                    let g_end = Some((g).fragment.borrow().end).unwrap().unwrap();
                    FragmentLinked {
                        fragment: Rc::new(RefCell::new(Fragment {
                            items,
                            start: f.fragment.borrow_mut().start,
                            end: Some(g_end),
                        })),
                        cross_link: start,
                    }
                };
                let key = f
                    .clone()
                    .fragment
                    .borrow_mut()
                    .end
                    .as_ref()
                    .unwrap()
                    .clone();
                stitch.fragment_by_start.insert(key, fg.clone());
                let key = fg.fragment.borrow_mut().end.clone().unwrap();
                fg.cross_link = end;
                stitch.fragment_by_end.insert(key, fg);
            } else {
                stitch
                    .fragment_by_start
                    .insert(f.fragment.borrow_mut().start.unwrap(), f.clone());
                stitch
                    .fragment_by_end
                    .insert(f.fragment.borrow_mut().end.unwrap(), f.clone());
            }
        } else if let Some(f) = stitch.fragment_by_start.get(&end) {
            let mut f = f.clone();
            let key = f.fragment.borrow_mut().start.as_ref().unwrap().clone();
            stitch.fragment_by_start.remove(&key);
            f.fragment.borrow_mut().items.push_front(*i);
            f.fragment.borrow_mut().start = Some(start);

            if let Some(g) = stitch.fragment_by_end.get(&start) {
                let g = g.clone();
                stitch
                    .fragment_by_end
                    .remove(g.fragment.borrow().end.as_ref().unwrap());

                let mut gf = if g == f {
                    f
                } else {
                    let g_then_f = g
                        .fragment
                        .borrow()
                        .items
                        .clone()
                        .into_iter()
                        .chain(f.fragment.borrow_mut().items.clone().into_iter());
                    FragmentLinked {
                        fragment: Rc::new(RefCell::new(Fragment {
                            items: VecDeque::from_iter(g_then_f),
                            start: g.fragment.borrow().start,
                            end: f.fragment.borrow_mut().end,
                        })),
                        cross_link: start,
                    }
                };

                let key = gf.fragment.borrow().start.unwrap();
                stitch.fragment_by_start.insert(key, gf.clone());
                let key = gf.fragment.borrow().end.unwrap();
                gf.cross_link = end;
                stitch.fragment_by_end.insert(key, gf);
            } else {
                let key = f.fragment.borrow_mut().start.unwrap();
                f.cross_link = end;
                stitch.fragment_by_start.insert(key, f.clone());
                let key = f.fragment.borrow_mut().end.unwrap();
                f.cross_link = start;
                stitch.fragment_by_end.insert(key, f);
            }
        } else {
            let f = Rc::new(RefCell::new(Fragment {
                items: VecDeque::from(vec![*i]),
                start: Some(start),
                end: Some(end),
            }));
            let f_linked1 = FragmentLinked {
                fragment: f.clone(),
                cross_link: end,
            };
            stitch.fragment_by_start.insert(start, f_linked1);
            let f_linked2 = FragmentLinked {
                fragment: f,
                cross_link: start,
            };
            stitch.fragment_by_end.insert(end, f_linked2);
        }
    });

    // dbg!(stitch.fragment_by_start.clone());
    // dbg!(stitch.fragment_by_end.clone());

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

    // dbg!(&fragments_plain);
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

/// Javascript allow shared pointers to become separated from the
/// underlying memory. Rust pointer must be valid at all times.
///
/// Regarding fragment_by_start and fragment_by_end.
///
/// When an index in fragment_by_start is deleted the corrsponding
/// index in fragment_by_end becomes undefined/empty.
///
/// To simulate this in Rust we need to cross_link the two together.
/// and when one is deleted the twin in the other list must also be deleted.
#[derive(Clone, Debug, PartialEq)]
struct FragmentLinked {
    pub cross_link: FragmentKey,
    fragment: Rc<RefCell<Fragment>>,
}

type FragmentKey = (i32, i32);

#[derive(Clone, Debug)]
struct Stitch<'a> {
    stitched_arcs: HashSet<usize>,
    fragment_by_start: BTreeMap<FragmentKey, FragmentLinked>,
    fragment_by_end: BTreeMap<FragmentKey, FragmentLinked>,
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
        fragment_by_end: &mut BTreeMap<FragmentKey, FragmentLinked>,
        fragment_by_start: &mut BTreeMap<FragmentKey, FragmentLinked>,
    ) {
        let search_iterator = fragment_by_end.keys().copied().collect::<Vec<(i32, i32)>>();
        for k in search_iterator {
            let f = fragment_by_end.get(&k).unwrap().clone();
            let cross_link = f.cross_link;

            fragment_by_start.remove(&k);
            fragment_by_end.remove(&cross_link.clone());

            let mut f = f.fragment.borrow_mut();
            f.start = None;
            f.end = None;
            for i in f.items.iter() {
                self.stitched_arcs.insert(translate(*i));
            }
            self.fragments.push(f.clone())
        }
        // dbg!("exit flush ");
        // dbg!(&self.fragments);
    }
}
