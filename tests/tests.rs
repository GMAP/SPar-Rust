use std::path::PathBuf;

#[test]
fn syntax() {
    let t = trybuild::TestCases::new();
    t.pass("tests/syntax/correct_syntax.rs");

    for p in PathBuf::from("tests/syntax/incorrect_syntax")
        .read_dir()
        .unwrap()
    {
        t.compile_fail(p.unwrap().path());
    }
}

#[test]
fn semantics() {
    let t = trybuild::TestCases::new();

    for p in PathBuf::from("tests/semantics").read_dir().unwrap() {
        t.pass(p.unwrap().path());
    }
}
