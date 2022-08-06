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
