#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
use arboard::SetExtLinux;

#[cfg(windows)]
use arboard::SetExtWindows;

#[cfg(target_os = "macos")]
use arboard::SetExtApple;

/// Trait to expose exclude from history functionality from [`arboard`] crate's `SetExt*` extensions.
/// On Linux, it calls [`arboard::SetExtLinux::exclude_from_history`]
/// On MacOS, it calls [`arboard::SetExtApple::exclude_from_history`]
/// On Windows, it calls [`arboard::SetExtWindows::exclude_from_history`], [`arboard::SetExtWindows::exclude_from_cloud`], and [`arboard::SetExtWindows::exclude_from_monitoring`]
pub trait ExcludeSecret<'clipboard> {
    fn exclude_secret(self) -> arboard::Set<'clipboard>;
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
impl<'clipboard> ExcludeSecret<'clipboard> for arboard::Set<'clipboard> {
    fn exclude_secret(self) -> arboard::Set<'clipboard> {
        self.exclude_from_history()
    }
}

#[cfg(windows)]
impl<'clipboard> ExcludeSecret<'clipboard> for arboard::Set<'clipboard> {
    fn exclude_secret(self) -> arboard::Set<'clipboard> {
        self.exclude_from_history()
            .exclude_from_cloud()
            .exclude_from_monitoring()
    }
}

#[cfg(target_os = "macos")]
impl<'clipboard> ExcludeSecret<'clipboard> for arboard::Set<'clipboard> {
    fn exclude_secret(self) -> arboard::Set<'clipboard> {
        self.exclude_from_history()
    }
}
