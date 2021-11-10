use std::ops::AddAssign;

use geo::CoordFloat;
use topojson::TransformParams;

/// The second parameter in used in the case of multiple delta encoded arcs
/// (see the tests below: If in doubt set to zero.)
pub type TransformFn<T> = Box<dyn FnMut(&[f64], usize) -> Vec<T>>;

/// Given a set of transform parameters return a tranform function that transforms
/// position into a Vec<f64>
pub fn generate<T>(tp: TransformParams) -> TransformFn<T>
where
    T: 'static + AddAssign<T> + CoordFloat,
{
    let mut x0 = T::zero();
    let mut y0 = T::zero();
    let kx: T = T::from(tp.scale[0]).expect("Could not convert Transform::scale.x");
    let ky: T = T::from(tp.scale[1]).expect("could not convert Transform::scale().y");

    let dx = T::from(tp.translate[0]).expect("Could not convert Transform.translate.x");
    let dy = T::from(tp.translate[1]).expect("Could not convert Transform.translate.y");

    Box::new(move |input: &[f64], i| -> Vec<T> {
        if i == 0 {
            x0 = T::zero();
            y0 = T::zero();
        }
        let mut j = 2;
        let n = input.len();
        let mut output = Vec::with_capacity(n);
        x0 += T::from(input[0]).unwrap();
        output.push(x0 * kx + dx);
        y0 += T::from(input[1]).unwrap();
        output.push(y0 * ky + dy);

        // Copy over all remaining point in the input vector?
        while j < n {
            output.push(T::from(input[j]).unwrap());
            j += 1;
        }
        output
    })
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod transform_tests {
    extern crate pretty_assertions;

    use super::*;

    // tape("topojson.transform(topology) returns the identity function if transform is undefined", function(test) {
    //   var transform = topojson.transform(null),
    //       point;
    //   test.equal(transform(point = {}), point);
    //   test.end();
    // });
    // #[test]
    // fn transform_returns_the_identity_function() {
    //     println!(
    //         "topojson.transform(topology) returns the identity function if transform is undefined"
    //     );
    //     let point;
    //     let transform = transform(None);
    //     assert_eq!(transform, None);
    // }

    // tape("topojson.transform(topology) returns a point-transform function if transform is defined", function(test) {
    //   var transform = topojson.transform({scale: [2, 3], translate: [4, 5]});
    //   test.deepEqual(transform([6, 7]), [16, 26]);
    //   test.end();
    // });
    #[test]
    fn returns_a_point_transform_function() {
        println!("topojson.transform(topology) returns a point-transform function if transform is defined");
        let mut transform = generate::<f64>(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });

        assert_eq!(transform(&vec![6_f64, 7_f64], 0), vec![16_f64, 26_f64]);
    }

    // This test does not need to be ported because rust handles mutability differently.
    // the input to transform() is imutable.
    //
    // tape("transform(point) returns a new point", function(test) {
    //   var transform = topojson.transform({scale: [2, 3], translate: [4, 5]}),
    //       point = [6, 7];
    //   test.deepEqual(transform(point), [16, 26]);
    //   test.deepEqual(point, [6, 7]);
    //   test.end();
    // });

    #[test]
    fn preserves_extra_dimensions() {
        println!("transform(point) preserves extra dimensions");
        let mut transform = generate::<f64>(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(
            transform(&vec![6_f64, 7_f64, 42_f64], 0),
            [16_f64, 26_f64, 42_f64]
        );
    }

    #[test]
    fn transforms_individual_points() {
        println!("transform(point) transforms individual points");
        let mut transform = generate::<f64>(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(&vec![1_f64, 2_f64], 0), vec![6_f64, 11_f64]);
        assert_eq!(transform(&vec![3_f64, 4_f64], 0), vec![10_f64, 17_f64]);
        assert_eq!(transform(&vec![5_f64, 6_f64], 0), vec![14_f64, 23_f64]);
    }

    #[test]
    fn transforms_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = generate::<f64>(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(&vec![1_f64, 2_f64], 0), vec![6_f64, 11_f64]);
        assert_eq!(transform(&vec![3_f64, 4_f64], 1), vec![12_f64, 23_f64]);
        assert_eq!(transform(&vec![5_f64, 6_f64], 2), vec![22_f64, 41_f64]);
        assert_eq!(transform(&vec![1_f64, 2_f64], 3), vec![24_f64, 47_f64]);
        assert_eq!(transform(&vec![3_f64, 4_f64], 4), vec![30_f64, 59_f64]);
        assert_eq!(transform(&vec![5_f64, 6_f64], 5), vec![40_f64, 77_f64]);
    }

    #[test]
    fn transforms_mutliple_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = generate::<f64>(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(&vec![1_f64, 2_f64], 0), vec![6_f64, 11_f64]);
        assert_eq!(transform(&vec![3_f64, 4_f64], 1), vec![12_f64, 23_f64]);
        assert_eq!(transform(&vec![5_f64, 6_f64], 2), vec![22_f64, 41_f64]);
        assert_eq!(transform(&vec![1_f64, 2_f64], 0), vec![6_f64, 11_f64]);
        assert_eq!(transform(&vec![3_f64, 4_f64], 1), vec![12_f64, 23_f64]);
        assert_eq!(transform(&vec![5_f64, 6_f64], 2), vec![22_f64, 41_f64]);
    }
}
