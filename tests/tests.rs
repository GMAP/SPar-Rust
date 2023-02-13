use std::path::PathBuf;

#[test]
fn semantics() {
    let t = trybuild::TestCases::new();

    for p in PathBuf::from("tests/semantics").read_dir().unwrap() {
        t.pass(p.unwrap().path());
    }
}
