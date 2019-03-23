//! Utilities to automatically extract updated packet/game object data from
//! the official flash ROTMG client, using an embedded build of
//! [rabcdasm](https://github.com/CyberShadow/RABCDAsm).

use crate::mappings::{Error as MappingError, Mappings};
use crate::net::packets::InternalPacketId;
use bimap::{BiHashMap, Overwritten};
use failure_derive::Fail;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::convert::From;
use std::fs::{read_to_string, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use tempfile::{tempdir, TempDir};

const ABCEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/abcexport"));
const RABCDASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rabcdasm"));
const SWFBINEXPORT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/swfbinexport"));

lazy_static! {
    static ref RC4_PATTERN: Regex = Regex::new(r#"\s+getlex\s+QName\(PackageNamespace\("com\.hurlant\.crypto"\),\s+"Crypto"\)\s+pushstring\s+"rc4"\s+getlex\s+QName\(PackageNamespace\("com\.company\.util"\),\s+"MoreStringUtil"\)\s+pushstring\s+"(\w+)"\s+pushbyte\s+0\s+pushbyte\s+26"#).unwrap();
    static ref PACKET_PATTERN: Regex = Regex::new(r#"trait const QName\(PackageNamespace\(""\), "(\w+)"\) slotid \d+ type QName\(PackageNamespace\(""\), "int"\) value Integer\((\d+)\) end"#).unwrap();
}

/// An error that occurred while extracting mappings from the game client
#[derive(Debug, Fail)]
pub enum Error {
    /// An IO error
    #[fail(display = "IO error: {}", _0)]
    IoError(IoError),

    /// An error converting the data to usable mappings
    #[fail(display = "Mapping error: {}", _0)]
    MappingError(MappingError),

    /// An error extracting data from the disassembled game client
    #[fail(display = "Extraction error: {}", _0)]
    ExtractionError(String),

    /// Some packets were left unmapped and `strict_packets` was specified.
    #[fail(display = "Some packets were unmapped - check logs")]
    UnmappedPackets,
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}

impl From<MappingError> for Error {
    fn from(e: MappingError) -> Self {
        Error::MappingError(e)
    }
}

/// A utility to extract embedded rabcdasm binaries and generate `Mappings`
/// from the official game client
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

    /// Extract mappings from the given SWF.
    ///
    /// If the `strict_packets` argument is true, an error will be returned
    /// if any packet IDs (either internal or from the game disassembly) are
    /// left unmapped. Otherwise, these will simply be ignored. In either case,
    /// a log message will be written when a packet is left unmapped.
    pub fn extract_mappings(&self, swf: &Path, strict_packets: bool) -> Result<Mappings, Error> {
        info!("Extracting game mappings from {}", swf.display());
        let abc = self.abcexport(swf)?;
        let code = self.rabcdasm(&abc)?;

        // extract RC4 keys
        let gsc_concrete = read_to_string(
            code.join("kabam/rotmg/messaging/impl/GameServerConnectionConcrete.class.asasm"),
        )?;

        let unified_rc4 = if let Some(matches) = RC4_PATTERN.captures(&gsc_concrete) {
            matches[1].to_string()
        } else {
            return Err(Error::ExtractionError(
                "Could not find RC4 keys".to_string(),
            ));
        };
        info!("Unified RC4 key: {}", unified_rc4);

        // extract packet IDs
        let packets = {
            let mut any_unmapped = false;

            // construct map of names to internal IDs
            let mut name_to_internal = HashMap::new();

            for (id, name) in InternalPacketId::get_name_mappings() {
                name_to_internal.insert(name.to_lowercase(), *id);
            }

            // read contents of GameServerConnection class
            let gsc = read_to_string(
                code.join("kabam/rotmg/messaging/impl/GameServerConnection.class.asasm"),
            )?;

            // construct map for game to internal ids
            let mut packet_mappings = BiHashMap::new();

            for cap in PACKET_PATTERN.captures_iter(&gsc) {
                let name = cap[1].replace('_', "").to_lowercase();
                let game_id = u8::from_str(&cap[2]).unwrap();

                if let Some(internal_id) = name_to_internal.remove(&name) {
                    debug!(
                        "Packet mapped: {:?} <> {}/{}",
                        internal_id, &cap[1], game_id
                    );
                    let overwritten = packet_mappings.insert(game_id, internal_id);
                    debug_assert_eq!(overwritten, Overwritten::Neither);
                } else {
                    warn!(
                        "No mapping found for packet {}/{} - skipping!",
                        &cap[1], game_id
                    );
                    any_unmapped = true;
                }
            }

            if !name_to_internal.is_empty() {
                for (_, v) in name_to_internal {
                    warn!("No match found for internal packet {:?} - skipping!", v);
                    any_unmapped = true;
                }
            }

            if any_unmapped && strict_packets {
                return Err(Error::UnmappedPackets);
            }

            packet_mappings
        };

        Ok(Mappings::new(unified_rc4, packets)?)
    }
}
