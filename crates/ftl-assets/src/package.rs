//! The only place that touches `ftldat`.
//!
//! `ftldat::Package` memory-maps the file and holds it through an `Rc`, so it
//! is neither `Send` nor long-lived here: `GameData::open` and `read_entry`
//! copy what they need and drop the package before they return.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::Error;

pub(crate) struct Package(ftldat::Package);

/// The first four bytes of a PKG file (FTL 1.6.1+). A DAT file starts with
/// its index size instead.
const PKG_SIGNATURE: &[u8; 4] = b"PKG\n";

pub(crate) fn open(path: &Path) -> Result<Package, Error> {
    let err = |detail: String| Error::Package {
        path: path.to_owned(),
        detail,
    };
    let mut head = [0u8; 4];
    let len = File::open(path)
        .and_then(|mut f| f.read_exact(&mut head).and(f.metadata()))
        .map_err(|e| err(e.to_string()))?
        .len();
    // A DAT file starts with its entry count, then one u32 offset per entry.
    // ftldat slices without bounds checks, so reject a count that cannot fit
    // before handing it over (the common case of a wrong file).
    let dat_index_end = 4 + 4 * u64::from(u32::from_le_bytes(head));
    if &head != PKG_SIGNATURE && dat_index_end > len {
        return Err(err("neither a PKG (FTL 1.6.1+) nor a DAT package".into()));
    }
    let pkg = if &head == PKG_SIGNATURE {
        ftldat::Package::from_path_pkg(path)
    } else {
        ftldat::Package::from_path_dat(path)
    }
    .map_err(|e| err(e.to_string()))?;
    Ok(Package(pkg))
}

impl Package {
    pub(crate) fn bytes(&self, inner_path: &str) -> Option<Vec<u8>> {
        self.0.content_by_path(inner_path)
    }

    pub(crate) fn inner_paths(&self) -> Vec<String> {
        self.0.inner_paths()
    }

    pub(crate) fn require(&self, inner_path: &str) -> Result<Vec<u8>, Error> {
        self.bytes(inner_path).ok_or_else(|| Error::Missing {
            inner_path: inner_path.to_owned(),
        })
    }
}
