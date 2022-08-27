use topojson::ArcIndexes;

/// A wrapped polygon which can be marked with a underscore to imply it has
/// been processed.
///
/// In javascript object are dynamic in rust we need this wrapper.
#[derive(Clone, Debug)]
pub struct PolygonU {
    pub v: Vec<ArcIndexes>,
    underscore: bool,
}

impl PolygonU {
    #[inline]
    pub(super) fn new(v: Vec<ArcIndexes>) -> Self {
        Self {
            v,
            underscore: false,
        }
    }

    #[inline]
    pub fn is_not_marked(&self) -> bool {
        !self.underscore
    }

    #[inline]
    pub fn mark(&mut self) {
        self.underscore = true;
    }

    #[inline]
    pub fn unmark(&mut self) {
        self.underscore = false;
    }
}
