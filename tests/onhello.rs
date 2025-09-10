use std::path::Path;

use snapbox::cmd::Command;
use snapbox::cmd::cargo_bin;
use snapbox::file;

#[test]
fn test () {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/hello");
    let _ = Command::new(cargo_bin("elfrust"))
        .arg(path)
        .assert()
        .stdout_eq(file!["./data/hello.output"])
        .stderr_eq("")
        .success();
    
}
