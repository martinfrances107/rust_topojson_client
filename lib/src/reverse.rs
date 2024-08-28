pub(crate) fn reverse<T>(array: &mut [T], n: usize)
where
    T: Clone,
{
    let mut j = array.len();
    let mut i = j - n;

    j -= 1;
    while i < j {
        let t = array[i].clone();
        array[i] = array[j].clone();
        i += 1;
        array[j] = t;
        j -= 1;
    }
}

#[cfg(test)]
mod reverse_tests {

    use super::*;
    use pretty_assertions::assert_eq;

    // There is no equivalent test in the javascript version.
    #[test]
    fn test_partial_reverse() {
        let mut a = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        reverse(&mut a, 5);
        assert_eq!(a, vec![1, 2, 3, 4, 5, 10, 9, 8, 7, 6]);

        let mut b = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        reverse(&mut b, 2);
        assert_eq!(b, vec![1, 2, 3, 4, 5, 6, 7, 8, 10, 9]);
    }
}

// Missing equivalant test code from javascript.
// TODO push this to original javascript module.
//
// let reverse = function (array, n) {
// 	var t, j = array.length, i = j - n;
// 	while (i < --j) t = array[i], array[i++] = array[j], array[j] = t;
// }

// let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
// let b = reverse(a, 5);
// console.log(a, b);

// let a2 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
// let b2 = reverse(a2, 2);
// console.log(a2, b2);
