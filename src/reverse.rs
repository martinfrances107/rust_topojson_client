use geo::CoordFloat;

pub(crate) fn reverse<T>(points: &mut Vec<(T, T)>, n: usize)
where
    T: CoordFloat,
{
    let mut t;
    let mut j = points.len();
    let mut i = j - n;

    while i < j {
        t = points[i];
        i += 1;
        points[i] = points[j];
        points[j] = t;
        j -= 1;
    }
}
