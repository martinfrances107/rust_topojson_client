use topojson::Position;
use topojson::TransformParams;

pub type TransformFn = Box<dyn FnMut(Position, Option<i32>) -> Vec<f64>>;

/// Given a set of transform parmaters return a tranformfunction that transform
/// position into a Vec<f64>
pub fn generate(tp: TransformParams) -> TransformFn {
    let mut x0 = 0_f64;
    let mut y0 = 0_f64;
    let kx = tp.scale[0];
    let ky = tp.scale[1];

    let dx = tp.translate[0];
    let dy = tp.translate[1];

    Box::new(move |input: Position, i| -> Vec<f64> {
        match i {
            Some(i) => {
                if i == 0 {
                    x0 = 0_f64;
                    y0 = 0_f64;
                }
            }
            None => {
                x0 = 0_f64;
                y0 = 0_f64;
            }
        }

        let mut j = 2;
        let n = input.len();
        let mut output = Vec::with_capacity(n);
        x0 += input[0];
        output.push(x0 * kx + dx);
        y0 += input[1];
        output.push(y0 * ky + dy);
        while j < n {
            output.push(input[j]);
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
        let mut transform = generate(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });

        assert_eq!(transform(vec![6_f64, 7_f64], None), [16_f64, 26_f64]);
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
        let mut transform = generate(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(
            transform(vec![6_f64, 7_f64, 42_f64], None),
            [16_f64, 26_f64, 42_f64]
        );
    }

    #[test]
    fn transforms_individual_points() {
        println!("transform(point) transforms individual points");
        let mut transform = generate(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(vec![1_f64, 2_f64], None), [6_f64, 11_f64]);
        assert_eq!(transform(vec![3_f64, 4_f64], None), [10_f64, 17_f64]);
        assert_eq!(transform(vec![5_f64, 6_f64], None), [14_f64, 23_f64]);
    }

    #[test]
    fn transforms_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = generate(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(vec![1_f64, 2_f64], Some(0)), [6_f64, 11_f64]);
        assert_eq!(transform(vec![3_f64, 4_f64], Some(1)), [12_f64, 23_f64]);
        assert_eq!(transform(vec![5_f64, 6_f64], Some(2)), [22_f64, 41_f64]);
        assert_eq!(transform(vec![1_f64, 2_f64], Some(3)), [24_f64, 47_f64]);
        assert_eq!(transform(vec![3_f64, 4_f64], Some(4)), [30_f64, 59_f64]);
        assert_eq!(transform(vec![5_f64, 6_f64], Some(5)), [40_f64, 77_f64]);
    }

    #[test]
    fn transforms_mutliple_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = generate(TransformParams {
            scale: [2_f64, 3_f64],
            translate: [4_f64, 5_f64],
        });
        assert_eq!(transform(vec![1_f64, 2_f64], Some(0)), [6_f64, 11_f64]);
        assert_eq!(transform(vec![3_f64, 4_f64], Some(1)), [12_f64, 23_f64]);
        assert_eq!(transform(vec![5_f64, 6_f64], Some(2)), [22_f64, 41_f64]);
        assert_eq!(transform(vec![1_f64, 2_f64], Some(0)), [6_f64, 11_f64]);
        assert_eq!(transform(vec![3_f64, 4_f64], Some(1)), [12_f64, 23_f64]);
        assert_eq!(transform(vec![5_f64, 6_f64], Some(2)), [22_f64, 41_f64]);
    }
}
