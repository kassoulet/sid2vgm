use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_conversion_andropolis() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--")
        .arg("tests/sids/Andropolis.sid")
        .arg("-o")
        .arg("tests/output.vgm")
        .arg("-d")
        .arg("2")
        .arg("--stats");

    let output = cmd.output().expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Conversion failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_path = PathBuf::from("tests/output.vgm");
    assert!(output_path.exists(), "Output file was not created");

    let metadata = fs::metadata(&output_path).expect("Failed to get metadata");
    assert!(metadata.len() > 64, "VGM file is too small (header only?)");

    // Check for "Vgm " ident
    let content = fs::read(&output_path).expect("Failed to read output");
    assert_eq!(&content[0..4], b"Vgm ", "Invalid VGM identifier");

    // Cleanup
    fs::remove_file(output_path).ok();
}

#[test]
fn test_vgz_conversion() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--")
        .arg("tests/sids/Andropolis.sid")
        .arg("-o")
        .arg("tests/output.vgz")
        .arg("-d")
        .arg("1");

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success());

    let output_path = PathBuf::from("tests/output.vgz");
    assert!(output_path.exists());

    // Check for GZIP header (0x1f 0x8b)
    let content = fs::read(&output_path).expect("Failed to read output");
    assert_eq!(content[0], 0x1f);
    assert_eq!(content[1], 0x8b);

    // Cleanup
    fs::remove_file(output_path).ok();
}
