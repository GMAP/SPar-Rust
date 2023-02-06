extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut vec: Vec<u32> = Vec::with_capacity(100000);
    for _ in 0..100000 {
        vec.push(rand::random());
    }

    let vec_slice = &mut vec[0..];
    let mut other_vec = Vec::new();
    to_stream!(INPUT(vec_slice), {
        for i in 1..10 {
            let split = vec_slice.split_at_mut(i * 10000);
            vec_slice = split.1;
            let input = split.0;
            STAGE(INPUT(input), OUTPUT(output), REPLICATE = 9 {
                input.sort();
                let output = input;
            });
            other_vec.push(output);
        }
    });

    assert_eq!(other_vec.len(), 9);
    for vec in other_vec {
        let mut cur = 0;
        for i in vec {
            assert!(cur <= *i);
            cur = *i;
        }
    }

    Ok(())
}
