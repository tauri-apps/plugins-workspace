// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Os<R>> {
    Ok(Os(app.clone()))
}

/// Access to the device APIs of the os plugin.
pub struct Os<R: Runtime>(#[allow(dead_code)] AppHandle<R>);

impl<R: Runtime> Os<R> {
    /// Returns information about the device, see [`DeviceInfo`].
    ///
    /// On desktop only [`DeviceInfo::model`], [`DeviceInfo::manufacturer`] and
    /// [`DeviceInfo::total_memory`] are read, the other fields are always `None`. This never fails
    /// on desktop: values that cannot be read are `None`.
    pub fn device_info(&self) -> crate::Result<DeviceInfo> {
        Ok(DeviceInfo {
            model: platform::model(),
            manufacturer: platform::manufacturer(),
            is_virtual: None,
            total_memory: platform::total_memory(),
            webview_version: None,
        })
    }

    /// Returns the name the user gave the device. Always `None` on desktop, see
    /// [`hostname`](crate::hostname) instead.
    pub fn device_name(&self) -> crate::Result<Option<String>> {
        Ok(None)
    }

    /// Returns the battery level and charging state. Always `None` on desktop.
    pub fn battery_info(&self) -> crate::Result<Option<BatteryInfo>> {
        Ok(None)
    }
}

#[cfg_attr(
    not(any(target_os = "linux", target_os = "macos", windows)),
    allow(dead_code)
)]
fn non_empty(value: impl AsRef<str>) -> Option<String> {
    let value = value
        .as_ref()
        .trim_matches(|c: char| c.is_whitespace() || c == '\0');
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(target_os = "linux")]
mod platform {
    use super::non_empty;

    fn read(path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok().and_then(non_empty)
    }

    pub fn model() -> Option<String> {
        read("/sys/class/dmi/id/product_name")
            // ARM boards such as the Raspberry Pi have no DMI table but a device tree
            .or_else(|| read("/sys/firmware/devicetree/base/model"))
    }

    pub fn manufacturer() -> Option<String> {
        read("/sys/class/dmi/id/sys_vendor")
    }

    pub fn total_memory() -> Option<u64> {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
        parse_mem_total(&meminfo)
    }

    pub(super) fn parse_mem_total(meminfo: &str) -> Option<u64> {
        let line = meminfo.lines().find(|l| l.starts_with("MemTotal:"))?;
        let kib: u64 = line
            .trim_start_matches("MemTotal:")
            .trim()
            .trim_end_matches("kB")
            .trim()
            .parse()
            .ok()?;
        kib.checked_mul(1024)
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::non_empty;

    fn sysctl(name: &std::ffi::CStr, buf: &mut [u8]) -> Option<usize> {
        let mut len = buf.len();
        // SAFETY: `buf` is valid for `len` bytes and `len` is updated with the written size.
        let ret = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                buf.as_mut_ptr().cast(),
                &mut len,
                std::ptr::null_mut(),
                0,
            )
        };
        (ret == 0).then_some(len)
    }

    pub fn model() -> Option<String> {
        let mut buf = [0u8; 256];
        let len = sysctl(c"hw.model", &mut buf)?;
        non_empty(String::from_utf8_lossy(&buf[..len]))
    }

    pub fn manufacturer() -> Option<String> {
        Some("Apple".into())
    }

    pub fn total_memory() -> Option<u64> {
        let mut buf = [0u8; 8];
        let len = sysctl(c"hw.memsize", &mut buf)?;
        (len == buf.len()).then(|| u64::from_ne_bytes(buf))
    }
}

#[cfg(windows)]
mod platform {
    use super::non_empty;
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::{
            Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW},
            SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        },
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn bios_value(name: &str) -> Option<String> {
        let key = wide(r"HARDWARE\DESCRIPTION\System\BIOS");
        let name = wide(name);
        let mut buf = [0u16; 256];
        let mut size = std::mem::size_of_val(&buf) as u32;
        // SAFETY: all strings are NUL-terminated and `buf` is valid for `size` bytes.
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let len = (size as usize / 2).min(buf.len());
        non_empty(String::from_utf16_lossy(&buf[..len]))
    }

    pub fn model() -> Option<String> {
        bios_value("SystemProductName")
    }

    pub fn manufacturer() -> Option<String> {
        bios_value("SystemManufacturer")
    }

    pub fn total_memory() -> Option<u64> {
        // SAFETY: MEMORYSTATUSEX is a plain C struct, all-zero is a valid value.
        let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        // SAFETY: `status` is a valid MEMORYSTATUSEX with `dwLength` set.
        (unsafe { GlobalMemoryStatusEx(&mut status) } != 0).then_some(status.ullTotalPhys)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod platform {
    pub fn model() -> Option<String> {
        None
    }

    pub fn manufacturer() -> Option<String> {
        None
    }

    pub fn total_memory() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_empty_trims_whitespace_and_nul() {
        assert_eq!(
            super::non_empty("Raspberry Pi 4\0").as_deref(),
            Some("Raspberry Pi 4")
        );
        assert_eq!(super::non_empty(" \n").as_deref(), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_mem_total() {
        let meminfo = "MemTotal:       16314892 kB\nMemFree:         1234 kB\n";
        assert_eq!(
            super::platform::parse_mem_total(meminfo),
            Some(16314892 * 1024)
        );
        assert_eq!(super::platform::parse_mem_total("MemFree: 1 kB"), None);
    }
}
