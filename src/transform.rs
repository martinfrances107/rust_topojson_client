use serde_json::Value;

pub type TransformFn = Box<dyn FnMut(Vec<i32>, Option<i32>) -> Vec<i32>>;

pub fn transform(transform: Option<Value>) -> TransformFn {
    match transform {
        None => {
            // identity transform
            Box::new(|x: Vec<i32>, _i32| x)
        }
        Some(t) => {
            let mut x0 = 0;
            let mut y0 = 0;
            let kx: i32 = t
                .pointer("/scale/0")
                .expect("must contain a scale array")
                .as_i64()
                .expect("must be an int")
                .try_into()
                .expect("cannot reduce to 32 bits.");
            let ky: i32 = t
                .pointer("/scale/1")
                .expect("must contain a scale array")
                .as_i64()
                .expect("must be an int")
                .try_into()
                .expect("cannot reduce to 32 bits.");
            let dx: i32 = t
                .pointer("/translate/0")
                .expect("must contain a translate array")
                .as_i64()
                .expect("must be an int")
                .try_into()
                .expect("cannot reduce to 32 bits.");
            let dy: i32 = t
                .pointer("/translate/1")
                .expect("must contain a translate array")
                .as_i64()
                .expect("must be an int")
                .try_into()
                .expect("cannot reduce to 32 bits.");

            Box::new(move |input, i| -> Vec<i32> {
                match i {
                    Some(i) => {
                        if i == 0 {
                            x0 = 0;
                            y0 = 0;
                        }
                    }
                    None => {
                        x0 = 0;
                        y0 = 0;
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
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod transform_tests {
    extern crate pretty_assertions;

    use super::*;

    use serde_json::json;

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
        let mut transform = transform(Some(json!({
            "scale": [2, 3],
            "translate": [4,5]
        })));
        assert_eq!(transform(vec![6, 7], None), [16, 26]);
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
        let mut transform = transform(Some(json!({
            "scale": [2, 3],
            "translate": [4,5]
        })));
        assert_eq!(transform(vec![6, 7, 42], None), [16, 26, 42]);
    }

    #[test]
    fn transforms_individual_points() {
        println!("transform(point) transforms individual points");
        let mut transform = transform(Some(json!({
            "scale": [2, 3],
            "translate": [4,5]
        })));
        assert_eq!(transform(vec![1, 2], None), [6, 11]);
        assert_eq!(transform(vec![3, 4], None), [10, 17]);
        assert_eq!(transform(vec![5, 6], None), [14, 23]);
    }

    #[test]
    fn transforms_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = transform(Some(json!({
            "scale": [2, 3],
            "translate": [4,5]
        })));
        assert_eq!(transform(vec![1, 2], Some(0)), [6, 11]);
        assert_eq!(transform(vec![3, 4], Some(1)), [12, 23]);
        assert_eq!(transform(vec![5, 6], Some(2)), [22, 41]);
        assert_eq!(transform(vec![1, 2], Some(3)), [24, 47]);
        assert_eq!(transform(vec![3, 4], Some(4)), [30, 59]);
        assert_eq!(transform(vec![5, 6], Some(5)), [40, 77]);
    }

    #[test]
    fn transforms_mutliple_delta_encoded_arcs() {
        println!("transform(point, index) transforms delta-encoded arcs");
        let mut transform = transform(Some(json!({
            "scale": [2, 3],
            "translate": [4,5]
        })));
        assert_eq!(transform(vec![1, 2], Some(0)), [6, 11]);
        assert_eq!(transform(vec![3, 4], Some(1)), [12, 23]);
        assert_eq!(transform(vec![5, 6], Some(2)), [22, 41]);
        assert_eq!(transform(vec![1, 2], Some(0)), [6, 11]);
        assert_eq!(transform(vec![3, 4], Some(1)), [12, 23]);
        assert_eq!(transform(vec![5, 6], Some(2)), [22, 41]);
    }
}
