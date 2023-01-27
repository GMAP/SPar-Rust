extern crate spar_rust;
use spar_rust::to_stream;

pub fn main() {
    let a = 1;
    let b = 2;
    to_stream!({});
    to_stream!(INPUT(a), {});
    to_stream!(INPUT(a, b), {});
    to_stream!(OUTPUT(a), {});
    to_stream!(OUTPUT(a, b), {});
    to_stream!(INPUT(a), OUTPUT(a), {});
    to_stream!(INPUT(a, b), OUTPUT(a, b), {});
    to_stream!(REPLICATE = 1, {});
    to_stream!(INPUT(a), REPLICATE = 1, {});
    to_stream!(OUTPUT(a), REPLICATE = 1, {});
    to_stream!(INPUT(a), OUTPUT(b), REPLICATE = 1, {});
    to_stream!(INPUT(a), {
        let c = 3;
        STAGE({
            println!("Stages");
        });
        STAGE(INPUT(a), {
            println!("MAKE SURE THE MACRO DOES NOT ERASE THIS");
            println!("TESTING TWO STATEMENTS");
        });
        STAGE(INPUT(a, c), {});
        STAGE(OUTPUT(c), {});
        STAGE(OUTPUT(a, c), {});
        STAGE(INPUT(a, c), OUTPUT(a, c), {});
        STAGE(REPLICATE = 1, {});
        STAGE(INPUT(a), REPLICATE = 1, {});
        STAGE(OUTPUT(a), REPLICATE = 1, {});
        STAGE(INPUT(a), OUTPUT(c), REPLICATE = 1, {});
    });
    to_stream!({
        for i in 0..3 {
            STAGE(INPUT(i), {
                println!("hi: {i}");
            });
        }
    });
    to_stream!({
        while true {
            for i in 0..3 {
                match i {
                    0 => println!("hey"),
                    1 => println!("hi"),
                    2 => {
                        for j in i..5 {
                            STAGE(INPUT(j), {
                                println!("hi from stage: {j}");
                            });
                        }
                    }
                    _ => println!("OH NO"),
                }
            }
        }
    });
}
