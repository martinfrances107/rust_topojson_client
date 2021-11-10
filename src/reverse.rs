use geo::CoordFloat;

pub(crate) fn reverse<T>(array: &mut Vec<T>, n: usize)
where
    T: CoordFloat,
{
    let mut t;
    let mut j = array.len();
    let mut i = j - n;

    while i < j {
        t = array[i];
        i += 1;
        array[i] = array[j];
        array[j] = t;
        j -= 1;
    }
}
