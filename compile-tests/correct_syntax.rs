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
}
