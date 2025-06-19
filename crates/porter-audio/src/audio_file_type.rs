use bincode::Decode;
use bincode::Encode;

use std::ffi::OsStr;

/// Represents a supported audio file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFileType {
    Wav,
    Flac,
}

impl AsRef<OsStr> for AudioFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            AudioFileType::Wav => OsStr::new("wav"),
            AudioFileType::Flac => OsStr::new("flac"),
        }
    }
}
