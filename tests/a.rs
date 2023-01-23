use spar_rust::to_stream;

#[test]
fn basic() {
    to_stream!({
        // Anything inside this block will be processed by the macro
    });
}
