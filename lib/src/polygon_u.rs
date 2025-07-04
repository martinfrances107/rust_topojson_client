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

impl From<Vec<ArcIndexes>> for PolygonU {
    fn from(v: Vec<ArcIndexes>) -> Self {
        Self {
            v,
            underscore: false,
        }
    }
}

impl PolygonU {
    #[deprecated(since = "0.2.0", note = "please use `from()` instead")]
    #[inline]
    pub(super) const fn new(v: Vec<ArcIndexes>) -> Self {
        Self {
            v,
            underscore: false,
        }
    }

    #[inline]
    pub const fn is_not_marked(&self) -> bool {
        !self.underscore
    }

    #[inline]
    pub const fn mark(&mut self) {
        self.underscore = true;
    }

    #[inline]
    pub const fn unmark(&mut self) {
        self.underscore = false;
    }
}
