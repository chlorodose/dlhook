#[dlhook::dlhook(origin = "open")]
fn open_hook(f: _, path: *const libc::c_char, flags: i32, mode: i32) -> i32 {
    let s = unsafe { std::ffi::CStr::from_ptr(path).to_str().unwrap() };
    if s.contains("home") {
        unsafe { std::ptr::write(libc::__errno_location(), libc::EPERM) };
        return -1;
    }
    unsafe { f(path, flags, mode) }
}
