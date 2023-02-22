extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for _ in 0..100000 {
        vec.push(rand::random());
    }

    let other_vec = Vec::new();
    let other_vec = to_stream!(INPUT(vec: Vec<u32>), OUTPUT(other_vec: Vec<Vec<u32>>), {
        let mut vec_slice = &mut vec[0..];
        for _ in 0..9 {
            let split = vec_slice.split_at_mut(10000);
            vec_slice = split.1;
            let input = split.0.to_vec();
            STAGE(INPUT(input: Vec<u32>), OUTPUT(sorted: Vec<u32>), REPLICATE = 9, {
                input.sort();
                let sorted = input;
            });
            STAGE(INPUT(sorted: Vec<u32>), OUTPUT(other_vec: Vec<Vec<u32>> ), {
                other_vec.push(sorted);
            });
        }
    });

    assert_eq!(other_vec.len(), 9);
    let mut counter: usize = 0;
    let mut cur: u32 = 0;
    for vec in other_vec {
        for i in vec {
            assert!(cur <= i, "cur: {cur}, i: {i}, at index: {counter}");
            cur = i;
            counter += 1;
        }
    }

    Ok(())
}
