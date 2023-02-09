extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let a = 1;

    let out = &mut 0;
    to_stream!(INPUT(a), {
        STAGE(INPUT(a), OUTPUT(b), {
            let mut a = a;
            for _ in 0..10 {
                a += 1;
            }
            let b = a;
        });
        *out = b;
    });

    assert_eq!(*out, a + 10);
    Ok(())
}