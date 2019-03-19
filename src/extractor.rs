//! Utilities to automatically extract updated packet/game object data from
//! the official flash ROTMG client, using an embedded build of
//! [rabcdasm](https://github.com/CyberShadow/RABCDAsm).

use crate::mappings::Mappings;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use std::fs::{read_to_string, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};

const ABCEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/abcexport"));
const RABCDASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rabcdasm"));
const SWFBINEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/swfbinexport"));

lazy_static! {
    static ref RC4_PATTERN: Regex = Regex::new(r#"\s+getlex\s+QName\(PackageNamespace\("com\.hurlant\.crypto"\),\s+"Crypto"\)\s+pushstring\s+"rc4"\s+getlex\s+QName\(PackageNamespace\("com\.company\.util"\),\s+"MoreStringUtil"\)\s+pushstring\s+"(\w+)"\s+pushbyte\s+0\s+pushbyte\s+26"#).unwrap();
}

pub struct Extractor {
    _dir: TempDir,
    abcexport: PathBuf,
    rabcdasm: PathBuf,
    swfbinexport: PathBuf,
}

impl Extractor {
    fn extract_binary(dir: &Path, name: &str, binary: &[u8]) -> IoResult<PathBuf> {
        // create the file
        let path = PathBuf::from(dir).join(name);
        let mut file = File::create(&path)?;

        // write the contents of the file
        file.write_all(binary)?;

        // only need to set executable permission on unix
        #[cfg(unix)]
        {
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

    /// Extract the embedded rabcdasm binaries as temporary files so they may
    /// be used
    pub fn extract() -> IoResult<Extractor> {
        // create a temporary directory
        let dir = tempdir()?;

        // extract the binaries
        let abcexport = Extractor::extract_binary(dir.path(), "abcexport", ABCEXPORT)?;
        let rabcdasm = Extractor::extract_binary(dir.path(), "rabcdasm", RABCDASM)?;
        let swfbinexport = Extractor::extract_binary(dir.path(), "swfbinexport", SWFBINEXPORT)?;

        // return struct
        Ok(Extractor {
            _dir: dir,
            abcexport,
            rabcdasm,
            swfbinexport,
        })
    }

    /// Run rabcdasm's `abcexport` command on the given swf, returning the
    /// path to the output file.
    fn abcexport(&self, swf: &Path) -> IoResult<PathBuf> {
        info!("Running abcexport on {}...", swf.display());
        let output = Command::new(&self.abcexport).arg(&swf).output()?;
        if output.status.success() {
            let mut name = swf.file_stem().unwrap().to_os_string();
            name.push("-0.abc");
            let path = swf.with_file_name(name);
            debug_assert!(path.exists(), "abcexport succeeded but .abc file not found");
            Ok(path)
        } else {
            Err(IoError::new(
                IoErrorKind::Other,
                format!(
                    "abcexport failed - stdout: {} stderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }

    /// Run `rabcdasm` on the given abc file, returning the path to the
    /// output directory.
    fn rabcdasm(&self, abc: &Path) -> IoResult<PathBuf> {
        info!("Running rabcdasm on {}...", abc.display());
        let output = Command::new(&self.rabcdasm).arg(&abc).output()?;
        if output.status.success() {
            let dir = abc.with_file_name(abc.file_stem().unwrap());
            debug_assert!(
                dir.exists(),
                "rabcdasm succeeded but output directory not found"
            );
            Ok(dir)
        } else {
            Err(IoError::new(
                IoErrorKind::Other,
                format!(
                    "rabcdasm failed - stdout: {} stderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }

    /// Extract mappings from the given SWF
    pub fn extract_mappings(&self, swf: &Path) -> IoResult<Mappings> {
        info!("Extracting game mappings from {}", swf.display());
        let abc = self.abcexport(swf)?;
        let code = self.rabcdasm(&abc)?;

        let gsc = read_to_string(
            code.join("kabam/rotmg/messaging/impl/GameServerConnectionConcrete.class.asasm"),
        )?;

        let unified_rc4 = if let Some(matches) = RC4_PATTERN.captures(&gsc) {
            matches[1].to_string()
        } else {
            return Err(IoError::new(IoErrorKind::Other, "could not find RC4 keys"));
        };
        info!("Unified RC4 key: {}", unified_rc4);

        if unified_rc4.len() != 52 {
            return Err(IoError::new(IoErrorKind::Other, "rc4 key length invalid"));
        }

        Ok(Mappings { unified_rc4 })
    }
}
