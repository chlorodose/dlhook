#![doc = include_str!("../README.md")]

/// Attr-macro to make `LD_PRELOAD` hooks
/// # Example
/// ```
/// #[dlhook::dlhook(origin = "getuid")]
/// fn fake_root_uid(_: _) -> u32 {
///     0
/// }
/// ```
pub use dlhook_macros::dlhook;

#[doc(hidden)]
pub mod __hidden {
    pub use libc::RTLD_NEXT;
    pub use libc::dlsym;
}
