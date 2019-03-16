//! Utilities to automatically extract updated packet/game object data from the flash ROTMG client

use std::fs::File;
use std::io::{Read, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

const ABCEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/abcexport"));
const RABCDASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rabcdasm"));
const SWFBINEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/swfbinexport"));

pub struct Binaries {
    dir: TempDir,
    abcexport: PathBuf,
    rabcdasm: PathBuf,
    swfbinexport: PathBuf,
}

impl Binaries {
    fn extract_binary(dir: &Path, name: &str, binary: &[u8]) -> IoResult<PathBuf> {
        // create the file
        let path = PathBuf::from(dir).join(name);
        let mut file = File::create(&path)?;

        // write the contents of the file
        file.write_all(binary)?;

        // only need to set executable permission on unix
        #[cfg(unix)] {
            use std::fs::{metadata, set_permissions};
            use std::os::unix::fs::PermissionsExt;

            // get current file permissions
            let mut perms = metadata(&path)?.permissions();

            // update permissions to allow user to execute
            perms.set_mode(perms.mode() | 0o100);

            // apply permissions
            set_permissions(&path, perms)?;
        }

        // return the path
        Ok(path)
    }

    /// Extract the embedded rabcdasm binaries as temporary files so they may be run
    pub fn extract() -> IoResult<Binaries> {
        // create a temporary directory
        let dir = tempdir()?;

        // extract the binaries
        let abcexport = Binaries::extract_binary(dir.path(), "abcexport", ABCEXPORT)?;
        let rabcdasm = Binaries::extract_binary(dir.path(), "rabcdasm", RABCDASM)?;
        let swfbinexport = Binaries::extract_binary(dir.path(), "swfbinexport", SWFBINEXPORT)?;

        // return struct
        Ok(Binaries {
            dir,
            abcexport,
            rabcdasm,
            swfbinexport,
        })
    }
}
