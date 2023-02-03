extern crate spar_rust;
use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let a = 1;

    let output = to_stream!(INPUT(a), OUTPUT(b), {
        STAGE(INPUT(a), OUTPUT(b), {
            let mut a = a;
            for _ in 0..10 {
                a += 1;
            }
            let b = a;
        });
        println!("b: {b}");
    });

    assert_eq!(output, a + 10);
    Ok(())
}
