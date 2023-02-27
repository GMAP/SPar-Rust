extern crate spar_rust;
extern crate rust_spp;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let a = 1;

    let stream_result = to_stream!(INPUT(a: u32), OUTPUT(u32), {
        STAGE(INPUT(a: u32), OUTPUT(u32), {
            let mut a = a;
            for _ in 0..10 {
                a += 1;
            }
            Some(a)
        });
    });

    assert_eq!(stream_result.iter().map(|i| *i).reduce(|acc, i| acc + i).unwrap(), 11);
    Ok(())
}
