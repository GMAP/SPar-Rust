extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for i in 0..10 {
        let min = (100000 * i) as f64;
        let max = (100000 * (i + 1)) as f64;
        for _ in 0..10000 {
            vec.push((rand::random::<f64>() * (max - min) + min) as u32);
        }
    }

    let mut result: Vec<u32> = Vec::new();
    to_stream!(INPUT(vec: Vec<u32>, result: Vec<u32>), ORDERED, {
        let mut vec_slice = &mut vec[0..];
        for _ in 0..10 {
            let split = vec_slice.split_at_mut(10000);
            vec_slice = split.1;
            let input = split.0.to_vec();
            STAGE(INPUT(input: Vec<u32>, result: Vec<u32>), REPLICATE = 9, {
                input.sort();
                result.extend(input);
            });
        }
    });

    assert_eq!(result.len(), 100000);
    let mut cur = 0;
    let mut index = 0;
    for v in result {
        assert!(cur <= v, "cur: {cur}, i: {v}, at index: {index}");
        cur = v;
        index += 1;
    }

    Ok(())
}
