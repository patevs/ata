use assert_cmd::Command;
use predicates::prelude::*;
use std::io::BufRead;
use transformrs::Provider;

fn ata() -> Command {
    Command::cargo_bin("ata").unwrap()
}

#[test]
fn unexpected_argument() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ata();
    cmd.arg("foobar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));

    Ok(())
}

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ata();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: ata"));

    Ok(())
}

/// Load a key from the local .env file.
///
/// This is used for testing only. Expects the .env file to contain keys for providers in the following format:
///
/// ```
/// DEEPINFRA_KEY="<KEY>"
/// OPENAI_KEY="<KEY>"
/// ```
fn load_key(provider: &Provider) -> String {
    fn finder(line: &Result<String, std::io::Error>, provider: &Provider) -> bool {
        line.as_ref().unwrap().starts_with(&provider.key_name())
    }
    let path = std::path::Path::new("test.env");
    let file = std::fs::File::open(path).expect("Failed to open .env file");
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();
    let key = lines.find(|line| finder(line, provider)).unwrap().unwrap();
    key.split("=").nth(1).unwrap().to_string()
}

#[test]
fn tts_no_args_output() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = ata();
    let key = load_key(&Provider::DeepInfra);
    cmd.arg("tts")
        .arg("--output")
        .arg("output.mp3")
        .env("DEEPINFRA_KEY", key)
        .write_stdin("Hello world")
        .current_dir(&dir)
        .assert()
        .success();

    let path = dir.path().join("output.mp3");
    assert!(path.exists());

    Ok(())
}

#[test]
fn tts_no_args() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = ata();
    let key = load_key(&Provider::DeepInfra);
    let cmd = cmd
        .arg("tts")
        .env("DEEPINFRA_KEY", key)
        .write_stdin("Hello world")
        .current_dir(&dir);
    let output = cmd.assert().success().get_output().stdout.clone();

    assert!(output.len() > 0);

    Ok(())
}
