use std::ffi::OsStr;

use serde::Deserialize;
use serde::Serialize;

/// Represents a supported model file type.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFileType {
    Obj,
    Smd,
    XnaLara,
    XModelExport,
    Cast,
    Maya,
    Fbx,
}

impl AsRef<OsStr> for ModelFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Obj => OsStr::new("obj"),
            Self::Smd => OsStr::new("smd"),
            Self::XnaLara => OsStr::new("mesh.ascii"),
            Self::XModelExport => OsStr::new("xmodel_export"),
            Self::Cast => OsStr::new("cast"),
            Self::Maya => OsStr::new("ma"),
            Self::Fbx => OsStr::new("fbx"),
        }
    }
}
