use geo::CoordFloat;

pub(crate) fn reverse<T>(array: &mut Vec<(T, T)>, n: usize)
where
    T: CoordFloat,
{
    let mut j = array.len();
    let mut i = j - n;

    j -= 1;
    while i < j {
        let t = array[i];
        array[i] = array[j];
        i += 1;
        array[j] = t;
        j -= 1;
    }
}
