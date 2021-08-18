//! Run scripts in `tests/ui/*.sh`, and compare their outputs with the
//! `.output` files.
//!
//! This code was adapted from the `bash_builtins` crate.

use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json as json;

/// Variable to send the path of the test file.
const TEST_FILE_VAR: &str = "TEST_FILE_PATH";

macro_rules! check_output {
    ($output:expr) => {
        if !$output.status.success() {
            std::io::stdout().write_all(&$output.stdout).unwrap();
            std::io::stderr().write_all(&$output.stderr).unwrap();
            panic!("command failed");
        }
    };
}

/// Invoke `cargo-build` and read its output in JSON format.
///
/// Cargo should send the path of the `.so` file in a `"compiler-artifact"`
/// message.
fn build() -> PathBuf {
    let build = Command::new(env::var("CARGO").unwrap())
        .arg("build")
        .arg("--quiet")
        .args(&["--message-format", "json"])
        .output()
        .unwrap();

    check_output!(build);

    for line in build.stdout.split(|c| *c == b'\n') {
        if let Ok(msg) = json::from_slice::<json::Value>(line) {
            if msg["reason"] == "compiler-artifact" {
                let target = &msg["target"];
                if target["kind"][0] == "cdylib" && target["name"] == env!("CARGO_PKG_NAME") {
                    if let Some(file) = msg["filenames"][0].as_str() {
                        return PathBuf::from(file);
                    }
                }
            }
        }
    }

    panic!("Unable to find the output path");
}

/// Create a shell script to launch the tests in a bash process.
///
/// Returns the path to the new file.
fn create_runner_file(target: &Path) -> PathBuf {
    let rc_path = target.join("init.sh");
    let mut output = BufWriter::new(File::create(&rc_path).unwrap());

    writeln!(
        &mut output,
        "
            exec 2>&1
            load_builtin() {{
                enable -f '{}' timehistory
            }}

            source ${}
        ",
        build().display(),
        TEST_FILE_VAR
    )
    .unwrap();

    rc_path
}

#[test]
fn check_ui() {
    let target = {
        let mut target = match env::var_os("CARGO_TARGET_DIR") {
            Some(t) => PathBuf::from(t),

            None => {
                let mut target = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
                target.push("target");
                target
            }
        };

        target.push("ui");
        target.push(env::var_os("CARGO_PKG_NAME").unwrap());

        std::fs::create_dir_all(&target).unwrap();

        target
    };

    let test_runner = create_runner_file(&target);

    let mut failed = 0;
    for source in fs::read_dir("tests/ui").unwrap() {
        let path = source.unwrap().path();

        if path.extension() != Some(OsStr::new("sh")) {
            continue;
        }

        let expected_output = {
            let mut exp_path = path.clone();
            exp_path.set_extension("output");
            fs::read_to_string(exp_path).unwrap_or_default()
        };

        let bash = Command::new("bash")
            .env("LC_ALL", "C")
            .env(TEST_FILE_VAR, &path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .arg(&test_runner)
            .spawn()
            .unwrap();

        // Wait until the script is done.
        let output = bash.wait_with_output().unwrap();
        check_output!(output);

        // Capture output and compare with the expected one.
        //
        // The path of the runner script is replaced with "$RUNNER".
        let test_output = String::from_utf8(output.stdout)
            .unwrap()
            .replace(test_runner.to_str().unwrap_or_default(), "$RUNNER");

        if test_output != expected_output {
            let test_name = path.file_name().unwrap();
            let mut output_copy = target.join(test_name);
            output_copy.set_extension("current-output");

            failed += 1;
            eprintln!("### {}: failed", path.display());
            eprintln!("=== OUTPUT ({})\n{}\n", output_copy.display(), test_output);
            eprintln!("=== EXPECTED\n{}\n", expected_output);

            fs::write(output_copy, test_output).unwrap();
        }
    }

    assert_eq!(failed, 0);
}
