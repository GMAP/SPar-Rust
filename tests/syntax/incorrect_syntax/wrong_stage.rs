extern crate spar_rust;
use spar_rust::to_stream;

pub fn main() {
    let a = 1;
    to_stream!(INPUT(a: u32), INPUT(a: u32), {
        STAGE();
    });
}
