extern crate spar_rust;
extern crate rust_spp;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let a = 1;

    let out = 0;
    let stream_result = to_stream!(INPUT(a: u32), OUTPUT(out: u32), {
        STAGE(INPUT(a: u32), OUTPUT(b: u32), {
            let mut a = a;
            for _ in 0..10 {
                a += 1;
            }
            let b = a;
        });
        STAGE(INPUT(b: u32), OUTPUT(out: u32), {
            out += b;
        });
    });

    assert_eq!(stream_result, 11);
    Ok(())
}
