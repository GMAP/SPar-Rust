extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let goal = 100000;
    let sum = (goal * goal + goal) / 2;
    let mut vec = Vec::new();
    for i in 1..=goal {
        vec.push(i as u64);
    }

    let mut result: Vec<Vec<u64>> = Vec::new();
    to_stream!(INPUT(vec: Vec<u64>, result: Vec<Vec<u64>>), {
        let mut vec_slice = &mut vec[0..];
        for _ in 0..10 {
            let split = vec_slice.split_at_mut(goal / 10);
            vec_slice = split.1;
            let input = split.0.to_vec();

            STAGE(
                INPUT(input: Vec<u64>),
                OUTPUT(input: Vec<u64>),
                REPLICATE = 9,
                {
                    for i in input.iter_mut() {
                        *i = *i + 1;
                    }
                },
            );

            STAGE(
                INPUT(input: Vec<u64>, result: Vec<Vec<u64>>),
                REPLICATE = 9,
                {
                    for i in input.iter_mut() {
                        *i = *i * 2;
                    }
                    result.push(input);
                },
            );
        }
    });

    assert_eq!(result.len(), 10);
    assert_eq!(
        result
            .iter()
            .map(|v| v.iter().map(|a| *a).reduce(|a, i| a + i).unwrap())
            .reduce(|acc, i| acc + i)
            .unwrap(),
        ((sum + goal) * 2) as u64
    );

    Ok(())
}
