extern crate spar_rust;
use spar_rust::to_stream;

pub fn main() {
    let a = 1;
    let b = 2;
    to_stream!({});
    to_stream!(INPUT(a: u32), {});
    to_stream!(INPUT(a: u32, b: u32), {});
    to_stream!(OUTPUT(a: u32), {});
    to_stream!(OUTPUT(a: u32, b: u32), {});
    to_stream!(INPUT(a: u32), OUTPUT(a: u32), {});
    to_stream!(INPUT(a: u32, b: u32), OUTPUT(a: u32, b: u32), {});
    to_stream!(REPLICATE = 1, {});
    to_stream!(INPUT(a: u32), REPLICATE = 1, {});
    to_stream!(OUTPUT(a: u32), REPLICATE = 1, {});
    to_stream!(INPUT(a: u32), OUTPUT(b: u32), REPLICATE = 1, {});
    to_stream!(INPUT(a: u32), {
        let c = 3;
        STAGE({
            println!("Stages");
        });
        STAGE(INPUT(a: u32), {
            println!("MAKE SURE THE MACRO DOES NOT ERASE THIS");
            println!("TESTING TWO STATEMENTS");
        });
        STAGE(INPUT(a: u32, c: u32), {});
        STAGE(OUTPUT(c: u32), {});
        STAGE(OUTPUT(a: u32, c: u32), {});
        STAGE(INPUT(a: u32, c: u32), OUTPUT(a: u32, c: u32), {});
        STAGE(REPLICATE = 1, {});
        STAGE(INPUT(a: u32), REPLICATE = 1, {});
        STAGE(OUTPUT(a: u32), REPLICATE = 1, {});
        STAGE(INPUT(a: u32), OUTPUT(c: u32), REPLICATE = 1, {});
    });
    to_stream!({
        for i in 0..3 {
            STAGE(INPUT(i: u32), {
                println!("hi: {i}");
            });
        }
    });
    to_stream!({
        let mut k = 10;
        while k > 0 {
            k -= 1;
            for i in 0..3 {
                match i {
                    0 => println!("hey"),
                    1 => println!("hi"),
                    2 => {
                        for j in i..5 {
                            STAGE(INPUT(j: u32), {
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
