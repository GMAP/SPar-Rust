extern crate rust_spp;
extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut a = 1;

    to_stream!(INPUT(a: u32), {
        STAGE(INPUT(a: u32), {
            for _ in 0..10 {
                a += 1;
            }
        });
    });

    assert_eq!(a, 11);
    Ok(())
}
