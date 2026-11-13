// SPDX-License-Identifier: RPL-1.5
//! Filesystem primitives (N-03, N-04, N-05, N-09), one contract per OS: a destination directory is opened once
//! and held; every file the engine creates, opens or renames is addressed RELATIVE to that held directory with a
//! single sanitized leaf, never through a re-resolved path, and a link planted at that leaf is refused, never
//! followed. Windows: `fsx_windows.rs` (NtCreateFile relative to the handle, no FILE_SHARE_DELETE, MOTW + AV via
//! IAttachmentExecute). macOS / Linux: `fsx_unix.rs` (openat with O_NOFOLLOW, renameat, quarantine / origin
//! extended attributes).

#[derive(Debug)]
pub enum FsError {
    /// the directory or a file is a junction/symlink (name surrogate) or the path resolved elsewhere (N-03)
    Reparse(String),
    Exists,
    NotFound,
    /// another handle still has the file open (e.g. a cancelled run's writer is closing)
    Busy,
    Io(String),
}
impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(windows)]
#[path = "fsx_windows.rs"]
mod imp;
#[cfg(unix)]
#[path = "fsx_unix.rs"]
mod imp;

pub use imp::*;
