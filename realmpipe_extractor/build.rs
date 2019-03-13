use git2::{Error as GitError, Repository};
use std::convert::From;
use std::fs::{read_dir, copy};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::env::{self, VarError as EnvError};

#[derive(Debug)]
enum BuildError {
    IoError(IoError),
    GitError(GitError),
    CompileError(ExitStatus),
    EnvError(EnvError),
}

impl From<IoError> for BuildError {
    fn from(e: IoError) -> Self {
        BuildError::IoError(e)
    }
}

impl From<GitError> for BuildError {
    fn from(e: GitError) -> Self {
        BuildError::GitError(e)
    }
}

impl From<EnvError> for BuildError {
    fn from(e: EnvError) -> Self {
        BuildError::EnvError(e)
    }
}

fn main() -> Result<(), BuildError> {
    // create a temporary directory to build rabcdasm in
    let dir = tempfile::tempdir()?;
    println!("Created temporary directory: {}", dir.as_ref().display());

    // clone the 1.18 version of rabcdasm
    let respository = Repository::clone("https://github.com/CyberShadow/RABCDAsm.git", &dir)?;
    println!("Successfully cloned repository");

    // build the project with dmd
    let exit_status = Command::new("dmd")
        .current_dir(&dir)
        .args(vec!["-run", "build_rabcdasm.d"])
        .spawn()?
        .wait()?;

    if !exit_status.success() {
        return Err(BuildError::CompileError(exit_status));
    }

    println!("Successfully compiled rabcdasm - directory contents:");

    for file in read_dir(&dir)? {
        println!("{}", file?.file_name().to_string_lossy());
    }

    let src_dir = PathBuf::from(dir.as_ref());
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // the rabcdasm binaries we need
    for name in &["abcexport", "rabcdasm", "swfbinexport"] {
        copy(src_dir.join(PathBuf::from(name)), out_dir.join(PathBuf::from(name)))?;
        println!("Copied {} binary", name);
    }

    Ok(())
}
