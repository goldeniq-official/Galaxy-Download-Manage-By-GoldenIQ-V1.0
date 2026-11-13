// SPDX-License-Identifier: RPL-1.5
//! macOS extended ACLs (council final review r2): the mode bits do not tell the whole story there, an ACL entry
//! can let another user add, delete or rename names in a folder whose mode is 0755. A folder whose ACL ALLOWS
//! writing, adding or deleting names is treated like a group/world-writable one. DENY entries (macOS puts
//! "everyone deny delete" on the home folders) take rights away and are fine. Shared with gdm-bridge by #[path].
use std::os::fd::{AsRawFd, BorrowedFd};
use std::os::raw::{c_int, c_void};

const ACL_TYPE_EXTENDED: c_int = 0x0000_0100;
const ACL_FIRST_ENTRY: c_int = 0;
const ACL_NEXT_ENTRY: c_int = -1;
const ACL_EXTENDED_ALLOW: c_int = 1;
// <sys/acl.h>: WRITE_DATA = ADD_FILE (1<<2), DELETE (1<<4), APPEND_DATA = ADD_SUBDIRECTORY (1<<5), DELETE_CHILD
// (1<<6), WRITE_ATTRIBUTES (1<<8), WRITE_EXTATTRIBUTES (1<<10), WRITE_SECURITY (1<<12), CHANGE_OWNER (1<<13)
const RISKY: [c_int; 8] = [1 << 2, 1 << 4, 1 << 5, 1 << 6, 1 << 8, 1 << 10, 1 << 12, 1 << 13];

extern "C" {
    fn acl_get_fd_np(fd: c_int, ty: c_int) -> *mut c_void;
    fn acl_get_entry(acl: *mut c_void, id: c_int, entry: *mut *mut c_void) -> c_int;
    fn acl_get_tag_type(entry: *mut c_void, tag: *mut c_int) -> c_int;
    fn acl_get_permset(entry: *mut c_void, set: *mut *mut c_void) -> c_int;
    fn acl_get_perm_np(set: *mut c_void, perm: c_int) -> c_int;
    fn acl_free(p: *mut c_void) -> c_int;
}

/// true when an ACL entry on this open file/folder ALLOWS someone to write, add, delete or re-own names
pub fn grants_write(fd: BorrowedFd<'_>) -> bool {
    unsafe {
        let acl = acl_get_fd_np(fd.as_raw_fd(), ACL_TYPE_EXTENDED);
        if acl.is_null() {
            return false; // no ACL at all
        }
        let mut risky = false;
        let mut e: *mut c_void = std::ptr::null_mut();
        let mut id = ACL_FIRST_ENTRY;
        while acl_get_entry(acl, id, &mut e) == 0 {
            id = ACL_NEXT_ENTRY;
            let mut tag = 0;
            let mut set: *mut c_void = std::ptr::null_mut();
            if acl_get_tag_type(e, &mut tag) != 0 || acl_get_permset(e, &mut set) != 0 {
                risky = true; // unreadable entry: assume the worst
                break;
            }
            if tag == ACL_EXTENDED_ALLOW && RISKY.iter().any(|p| acl_get_perm_np(set, *p) == 1) {
                risky = true;
                break;
            }
        }
        acl_free(acl);
        risky
    }
}
