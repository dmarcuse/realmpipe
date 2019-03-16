use git2::Repository;
use std::convert::From;
use std::env;
use std::fs::{copy, read_dir};
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // we don't need to recompile rabcdasm unless the build script changes
    println!("cargo:rerun-if-changed=build.rs");

    // create a temporary directory to build rabcdasm in
    let dir = tempfile::tempdir().expect("error creating temp dir");
    println!("Created temporary directory: {}", dir.as_ref().display());

    // clone rabcdasm
    Repository::clone("https://github.com/CyberShadow/RABCDAsm.git", &dir)
        .expect("error cloning repo");
    println!("Successfully cloned repository");

    // build the project with dmd
    let exit_status = Command::new("dmd")
        .current_dir(&dir)
        .args(vec!["-run", "build_rabcdasm.d"])
        .spawn()
        .expect("error starting d compiler")
        .wait()
        .expect("error waiting for d compiler");

    if !exit_status.success() {
        panic!("error compiling rabcdasm: {}", exit_status);
    }

    println!("Successfully compiled rabcdasm - directory contents:");

    for file in read_dir(&dir).unwrap() {
        println!("{}", file.unwrap().file_name().to_string_lossy());
    }

    let src_dir = PathBuf::from(dir.as_ref());
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("error getting OUT_DIR variable"));

    // copy the rabcdasm binaries we need
    for &name in &["abcexport", "rabcdasm", "swfbinexport"] {
        let src = src_dir.join(if cfg!(not(windows)) {
            PathBuf::from(name)
        } else {
            PathBuf::from(String::from(name) + ".exe")
        });

        copy(&src, out_dir.join(PathBuf::from(name))).expect(&format!(
            "error copying {} binary from {}",
            name,
            src.display()
        ));
        println!("Copied {} binary", name);
    }
}
