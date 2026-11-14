//! Where the core keeps its files. `CorePaths` and the path helpers live in the
//! foundation crate (#854); this hands them on as `crate::paths`. Shells pass a
//! `CorePaths` to `Core::open`, so its uniffi record is declared here, as a
//! remote type, and the bindings keep it in the core's namespace.

pub use anibeam_foundation::paths::*;

// uniffi's remote form restates the record's fields: it builds and reads the struct
// field by field, so a field this list lacks, adds or types differently fails to
// compile. The doc below is the bindings' docstring, kept as it was on the core's type.
/// Where the core keeps its files. Every field is a directory path as a
/// String, per the contract's closed type set; the accessors return PathBuf
/// for the code inside the crate.
#[uniffi::remote(Record)]
pub struct CorePaths {
    pub data_dir: String,
    pub config_dir: String,
    pub cache_dir: String,
    pub state_dir: String,
}
