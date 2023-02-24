extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for _ in 0..100000 {
        vec.push(rand::random());
    }

    let other_vec = Vec::new();
    let other_vec = to_stream!(INPUT(vec: Vec<u32>), OUTPUT(other_vec: Vec<u32>), {
        let mut vec_slice = &mut vec[0..];
        for _ in 0..9 {
            let split = vec_slice.split_at_mut(10000);
            vec_slice = split.1;
            let input = split.0.to_vec();
            STAGE(INPUT(input: Vec<u32>), OUTPUT(sorted: Vec<u32>), REPLICATE = 9, {
                input.sort();
                let sorted = input;
            });
            STAGE(INPUT(sorted: Vec<u32>), OUTPUT(other_vec: Vec<u32>), {
                other_vec.extend(&sorted);
            });
        }
    });

    assert_eq!(other_vec.len(), 90000);
    let mut cur: u32 = 0;
    for (index, i) in other_vec.iter().enumerate() {
        assert!(cur <= *i, "cur: {cur}, i: {i}, at index: {index}");
        cur = *i;
    }

    Ok(())
}
