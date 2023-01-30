use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

fn try_compile(source: &Path) -> bool {
    match std::process::Command::new("rustc")
        .arg("+nightly")
        .arg("-Zunpretty=expanded") // This is necessary so that rust tries to expand the macros
        .arg("-Zparse-only") // This means we won't generate a binary
        .arg(source.to_string_lossy().to_string())
        .arg("--extern")
        .arg("spar_rust=target/debug/libspar_rust.so")
        // NOTE: comment these out to see the diagnostic messages when compilation fails
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
    {
        Ok(exit_status) => exit_status.success(),
        Err(e) => {
            eprintln!("failed to execute rustc: {e}");
            false
        }
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    let path = match args.next() {
        Some(arg) => PathBuf::from(arg),
        None => return Err(format!("usage: {program_name} <crate top-level directory>")),
    };

    let mut correct_syntax = path.clone();
    correct_syntax.push(Path::new("tests/syntax/syntax-tests/correct_syntax.rs"));
    assert!(try_compile(&correct_syntax), "correct syntax failed to compile");

    let mut incorrect_syntax = path;
    incorrect_syntax.push(Path::new("tests/syntax/syntax-tests/incorrect_syntax"));
    let files = incorrect_syntax.read_dir().unwrap();
    for file in files {
        let file = file.unwrap();
        assert!(
            !try_compile(&file.path()),
            "{} compiled, when it shouldn't!",
            file.path().to_string_lossy()
        );
    }

    Ok(())
}
