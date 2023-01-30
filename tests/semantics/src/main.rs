use std::path::PathBuf;

use spar_rust::to_stream;

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    let _path = match args.next() {
        Some(arg) => PathBuf::from(arg),
        None => return Err(format!("usage: {program_name} <crate top-level directory>")),
    };

    let a = 1;

    to_stream!(INPUT(a), {
        STAGE(INPUT(a), OUTPUT(b), {
            let mut a = a;
            for _ in 0..10 {
                a += 1;
            }
            b = a;
        });
    });

    Ok(())
}
