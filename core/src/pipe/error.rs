use crate::proxy::codec::CodecError;
use failure_derive::Fail;
use std::convert::From;
use std::io::Error as IoError;

/// An error that occurred while setting up or using a connection betweeen the
/// client and server
#[derive(Debug, Fail)]
pub enum PipeError {
    /// An error reading or writing a packet
    #[fail(display = "codec error: {}", _0)]
    CodecError(CodecError),

    /// A generic IO error
    #[fail(display = "io error: {}", _0)]
    IoError(IoError),
}

impl From<CodecError> for PipeError {
    fn from(e: CodecError) -> Self {
        PipeError::CodecError(e)
    }
}

impl From<IoError> for PipeError {
    fn from(e: IoError) -> Self {
        PipeError::IoError(e)
    }
}
