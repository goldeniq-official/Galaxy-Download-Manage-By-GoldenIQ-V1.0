// SPDX-License-Identifier: RPL-1.5
//! macOS / Linux: the per-user Unix socket (B-02 on Unix). The socket lives in a folder only this user can enter
//! (0700, owned by the user, not a symlink); before a byte is sent the client checks that the process on the other
//! end runs as this user AND is the expected gdm executable (Linux SO_PEERCRED + /proc/<pid>/exe, macOS
//! LOCAL_PEERPID + proc_pidpath), as the Windows client checks the pipe server's image. One request per
//! connection: the client writes the message and shuts its write side; the reply is read to EOF (<= MAX_MSG).
//! Known limit (as on Windows, same-user processes are outside the threat model): the pid is read once per
//! connection; a peer that hands its socket to another process is not detected.
use crate::MAX_MSG;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// the private folder that holds the socket
pub fn endpoint_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/nonexistent"));
    #[cfg(target_vendor = "apple")]
    {
        home.join("Library/Application Support/GDM/run")
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        match std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).filter(|d| d.is_absolute()) {
            Some(d) => d.join("gdm"),
            None => home.join(".local/state/gdm/run"),
        }
    }
}

/// the socket path (kept under this name: callers share it with the Windows pipe name)
pub fn pipe_name() -> String {
    endpoint_dir().join("gdm.sock").display().to_string()
}

/// create (server) or check (client) the private folder: a real directory, owned by us, mode 0700
pub fn private_dir(create: bool) -> Result<PathBuf, String> {
    let d = endpoint_dir();
    if create {
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let m = std::fs::symlink_metadata(&d).map_err(|e| format!("no GDM: {e}"))?;
    if !m.is_dir() || m.uid() != uid() {
        return Err(format!("refused: {} is not a folder of this user", d.display()));
    }
    if m.permissions().mode() & 0o077 != 0 {
        if !create {
            return Err(format!("refused: {} is open to other users", d.display()));
        }
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    }
    Ok(d)
}

pub fn uid() -> u32 {
    unsafe { libc::getuid() }
}

/// (uid, pid) of the process at the other end of a connected Unix socket, as the kernel reports them
pub fn peer(s: &UnixStream) -> Option<(u32, u32)> {
    let fd = s.as_raw_fd();
    #[cfg(not(target_vendor = "apple"))]
    unsafe {
        let mut c: libc::ucred = std::mem::zeroed();
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        (libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_PEERCRED, &mut c as *mut _ as *mut _, &mut len) == 0 && c.pid > 0)
            .then_some((c.uid, c.pid as u32))
    }
    #[cfg(target_vendor = "apple")]
    unsafe {
        let (mut u, mut g) = (0, 0);
        if libc::getpeereid(fd, &mut u, &mut g) != 0 {
            return None;
        }
        let mut pid: libc::pid_t = 0;
        let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
        (libc::getsockopt(fd, libc::SOL_LOCAL, libc::LOCAL_PEERPID, &mut pid as *mut _ as *mut _, &mut len) == 0 && pid > 0)
            .then_some((u, pid as u32))
    }
}

/// the executable of a process
pub fn image_of(pid: u32) -> Option<PathBuf> {
    #[cfg(not(target_vendor = "apple"))]
    {
        std::fs::read_link(format!("/proc/{pid}/exe")).ok()
    }
    #[cfg(target_vendor = "apple")]
    unsafe {
        let mut buf = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        let n = libc::proc_pidpath(pid as i32, buf.as_mut_ptr() as *mut _, buf.len() as u32);
        (n > 0).then(|| PathBuf::from(String::from_utf8_lossy(&buf[..n as usize]).into_owned()))
    }
}

/// paths compare exactly on Unix (case matters), after resolving symlinks in the install location
pub fn norm(p: &Path) -> String {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).display().to_string()
}

/// B-02: connect, then check the server process (this user, the expected executable) BEFORE sending a byte
pub fn call(sock: &str, expected: &Path, msg: &[u8], wait: Duration) -> Result<Vec<u8>, String> {
    private_dir(false)?;
    let deadline = Instant::now() + wait;
    let mut s = loop {
        match UnixStream::connect(sock) {
            Ok(s) => break s,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)), // gdm still starting
            Err(e) => return Err(format!("no GDM: {e}")),
        }
    };
    let (u, pid) = peer(&s).ok_or("refused: peer unknown")?;
    if u != uid() {
        return Err(format!("refused: socket served by uid {u}"));
    }
    let image = image_of(pid).ok_or("refused: peer image unknown")?;
    if norm(&image) != norm(expected) {
        return Err(format!("refused: socket server is {}", image.display()));
    }
    s.set_read_timeout(Some(Duration::from_secs(30))).map_err(|e| e.to_string())?;
    s.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    s.write_all(msg).map_err(|e| format!("write: {e}"))?;
    s.shutdown(std::net::Shutdown::Write).map_err(|e| format!("write: {e}"))?;
    let mut buf = Vec::new();
    s.take(MAX_MSG as u64 + 1).read_to_end(&mut buf).map_err(|e| format!("read: {e}"))?;
    if buf.len() > MAX_MSG as usize {
        return Err("read: too large".into());
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// B-02 on Unix: a socket served by another executable never receives the payload; the genuine one does
    #[test]
    fn b02_refuses_a_socket_served_by_another_image() {
        let d = std::env::temp_dir().join(format!("gdm-ipc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", &d);
        std::env::set_var("HOME", &d);
        private_dir(true).unwrap();
        let sock = pipe_name();
        let l = std::os::unix::net::UnixListener::bind(&sock).unwrap();
        let got = std::sync::Arc::new(std::sync::Mutex::new(Vec::<usize>::new()));
        let g2 = got.clone();
        let srv = std::thread::spawn(move || {
            for c in l.incoming().take(2) {
                let mut c = c.unwrap();
                let mut b = Vec::new();
                if c.read_to_end(&mut b).is_ok() && !b.is_empty() {
                    g2.lock().unwrap().push(b.len());
                    let _ = c.write_all(b"{\"ok\":true}");
                }
            }
        });
        let e = call(&sock, Path::new("/usr/bin/gdm"), b"secret cookie", Duration::from_secs(2)).unwrap_err();
        assert!(e.starts_with("refused"), "{e}");
        let me = std::env::current_exe().unwrap();
        assert_eq!(call(&sock, &me, b"hello", Duration::from_secs(2)).unwrap(), b"{\"ok\":true}");
        srv.join().unwrap();
        assert_eq!(*got.lock().unwrap(), vec![5], "only the verified call reached the server");
        std::fs::set_permissions(endpoint_dir(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(call(&sock, &me, b"x", Duration::from_millis(100)).unwrap_err().starts_with("refused"), "a folder open to others is refused");
    }
}
