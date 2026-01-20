pub use dlhook_macros::dlhook;

#[doc(hidden)]
pub mod __hidden {
    pub use libc::RTLD_NEXT;
    pub use libc::dlsym;
}
