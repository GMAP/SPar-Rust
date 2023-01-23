extern crate spar_rust;
use spar_rust::to_stream;

pub fn main() {
    to_stream!(REPLICATE = 1, REPLICATE = 2, {});
}
