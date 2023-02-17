extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for _ in 0..100000 {
        vec.push(rand::random());
    }

    let vec_slice = &mut vec[0..];
    let mut other_vec = Vec::new();
    to_stream!(INPUT(vec_slice: &mut [u32]), OUTPUT(other_vec: Vec<u32>), {
        for _ in 0..9 {
            let split = vec_slice.split_at_mut(10000);
            vec_slice = split.1;
            let input = split.0;
            STAGE(INPUT(input: &mut [u32]), OUTPUT(sorted: &[u32]), REPLICATE = 9, {
                input.sort();
                let sorted = input;
            });
            STAGE(INPUT(sorted: &[u32]), OUTPUT(other_vec: Vec<u32> ), {
                other_vec.push(sorted);
            });
        }
    });

    assert_eq!(other_vec.len(), 9);
    let mut counter = 0;
    let mut cur = 0;
    for vec in other_vec {
        for i in vec {
            assert!(cur <= *i, "cur: {cur}, i: {i}, at index: {counter}");
            cur = *i;
            counter += 1;
        }
    }

    Ok(())
}
