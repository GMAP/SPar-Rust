extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for _ in 0..100000 {
        vec.push(rand::random());
    }

    let vec_slice = &mut vec[0..];
    type SLICE<'a> = &'a mut[u32];
    let mut other_vec = Vec::new();
    to_stream!(INPUT(vec_slice: SLICE), {
        for _ in 0..9 {
            let split = vec_slice.split_at_mut(10000);
            vec_slice = split.1;
            let input = split.0;
            STAGE(INPUT(input: SLICE), OUTPUT(output: SLICE), REPLICATE = 9, {
                input.sort();
                let output = input;
            });
            other_vec.push(output);
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
