/// Returns the bisection index for a slice of arc indexes.
///
/// Performs binary search.
///
/// # Arguments
///
/// * 'a' - A slice of arc indexes.
/// * 'x' - The threshold value use to separate into two halves.
///
pub fn bisect(a: &[i32], x: i32) -> usize {
    let mut lo = 0;
    let mut hi = a.len();

    while lo < hi {
        let mid = (lo + hi) >> 1;
        if a[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod bisect_test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple() {
        assert_eq!(bisect(&[0, 2, 3, 4], 4), 3);
        assert_eq!(bisect(&[1, 2, 3, 4], 3), 2);
    }
}
