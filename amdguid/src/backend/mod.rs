#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "xorg")]
pub mod xorg;

#[cfg(feature = "wayland")]
pub use wayland::run_app;

#[cfg(feature = "xorg")]
pub use xorg::run_app;
