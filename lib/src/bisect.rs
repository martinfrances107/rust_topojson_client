// pub fn bisect(a: Vec<usize>, x: usize) -> usize {
//     let lo = 0;
//     let hi = a.len();

//     while lo < hi {
//         let mid = lo + hi >> 1;
//         if a[mid] < x {
//             lo = mid + 1;
//         } else {
//             hi = mid;
//         }
//     }
//     lo
// }
