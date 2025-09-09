#![doc = include_str!("../README.md")]
use std::{env, path::PathBuf};

// TODO: Can we somehow guarantee that this directory exists?
/// Returns the path to the `localdata` directory.
///
/// The content of this directory will be persisted when either the app or the firmware are
/// upgraded. It is owned and writeable by `<APPUSR>`. As such, it is the recommended location for
/// configuration data.
///
/// See the [acap-documentation] for more information.
///
/// [acap-documentation]: https://axiscommunications.github.io/acap-documentation/docs/develop/application-project-structure.html#local-data
pub fn localdata_dir() -> Option<PathBuf> {
    Some(env::current_dir().ok()?.join("localdata"))
}

/// Returns the path to the directory containing _additional files_.
///
/// <div class="warning">
///     The existence and properties of this directory are undocumented.
/// </div>
pub fn additional_files_dir() -> Option<PathBuf> {
    app_dir()
}

/// Returns the path to the `html` directory.
///
/// <div class="warning">
///     The existence and properties of this directory are undocumented.
/// </div>
pub fn html_dir() -> Option<PathBuf> {
    app_dir().map(|d| d.join("html"))
}

/// Returns the path to the `lib` directory.
///
/// <div class="warning">
///     The existence and properties of this directory are undocumented.
/// </div>
pub fn lib_dir() -> Option<PathBuf> {
    app_dir().map(|d| d.join("lib"))
}

/// Returns the path to the directory into which the application has been unpacked.
///
/// That it works this way is not documented anywhere, but it appears to be true in practice.
fn app_dir() -> Option<PathBuf> {
    Some(env::current_exe().ok()?.parent()?.to_path_buf())
}
