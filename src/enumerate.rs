use std::io;

/// List all bridge interfaces on the system.
///
/// Uses `if_nameindex(2)` to enumerate network interfaces, then filters
/// for names starting with "bridge".
///
/// # Errors
///
/// Returns an error if `if_nameindex(2)` fails.
pub fn list_bridges() -> io::Result<Vec<String>> {
    // SAFETY: if_nameindex() returns a pointer to a NULL-terminated array
    // of if_nameindex structs, or NULL on failure. We free it with
    // if_freenameindex() after copying the names we need.
    let nameindex = unsafe { libc::if_nameindex() };
    if nameindex.is_null() {
        return Err(io::Error::last_os_error());
    }

    let mut bridges = Vec::new();
    let mut ptr = nameindex;

    // SAFETY: we iterate until we hit the sentinel entry (if_index == 0,
    // if_name == null). Each entry's if_name is a valid C string.
    unsafe {
        loop {
            let entry = &*ptr;
            if entry.if_index == 0 && entry.if_name.is_null() {
                break;
            }
            if !entry.if_name.is_null() {
                let name = std::ffi::CStr::from_ptr(entry.if_name);
                if let Ok(s) = name.to_str()
                    && s.starts_with("bridge")
                {
                    bridges.push(s.to_owned());
                }
            }
            ptr = ptr.add(1);
        }

        libc::if_freenameindex(nameindex);
    }

    Ok(bridges)
}
