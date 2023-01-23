use std::{path::Path, process::Stdio};

const DEPS_PATH: &'static str = "target/debug/deps";

fn try_compile(source: &Path) -> bool {
    let deps = Path::new(DEPS_PATH);
    let mut spar_dep = deps.to_path_buf();
    for file in deps.read_dir().unwrap() {
        let file = file.unwrap();
        let filename = file.file_name();
        let filename = filename.to_string_lossy();
        if filename.starts_with("libspar_rust") && filename.ends_with(".so") {
            spar_dep.push(filename.to_string());
        }
    }

    match std::process::Command::new("rustc")
        .arg(source.to_string_lossy().to_string())
        .arg("--out-dir")
        .arg("tmp")
        .arg("--extern")
        .arg("spar_rust=".to_string() + spar_dep.to_str().unwrap())
        // NOTE: change these to see the diagnostic messages when compilation fails
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

#[test]
fn should_compile() {
    assert!(try_compile(Path::new("compile-tests/correct_syntax.rs")))
}

#[test]
fn should_not_compile() {
    let files = Path::new("compile-tests/incorrect_syntax")
        .read_dir()
        .unwrap();
    for file in files {
        let file = file.unwrap();
        assert!(!try_compile(&file.path()));
    }
}
