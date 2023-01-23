extern crate spar_rust;
use spar_rust::to_stream;

pub fn main() {
    let a = 1;
    to_stream!(OUTPUT(a), OUTPUT(a), {});
}
