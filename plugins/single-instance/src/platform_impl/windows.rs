// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::SingleInstanceCallback;
use std::ffi::CStr;
use tauri::{
    AppHandle, Manager, RunEvent, Runtime,
    plugin::{self, TauriPlugin},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM,
        LRESULT, WIN32_ERROR, WPARAM,
    },
    System::{
        DataExchange::COPYDATASTRUCT,
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateMutexW, ReleaseMutex},
    },
    UI::WindowsAndMessaging::{
        self as w32wm, AllowSetForegroundWindow, CREATESTRUCTW, CreateWindowExW, DefWindowProcW,
        DestroyWindow, FindWindowW, GWL_STYLE, GWLP_USERDATA, GetWindowThreadProcessId,
        RegisterClassExW, SendMessageW, WINDOW_LONG_PTR_INDEX, WM_COPYDATA, WM_CREATE, WM_DESTROY,
        WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
        WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
    },
};

const WMCOPYDATA_SINGLE_INSTANCE_DATA: usize = 1542;

struct MutexHandle(isize);

struct TargetWindowHandle(isize);

struct UserData<R: Runtime> {
    app: AppHandle<R>,
    callback: Box<SingleInstanceCallback<R>>,
}

impl<R: Runtime> UserData<R> {
    unsafe fn from_hwnd_raw(hwnd: HWND) -> *mut Self {
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Self
    }

    unsafe fn from_hwnd<'a>(hwnd: HWND) -> &'a mut Self {
        &mut *Self::from_hwnd_raw(hwnd)
    }

    fn run_callback(&mut self, args: Vec<String>, cwd: String) {
        (self.callback)(&self.app, args, cwd)
    }
}

pub fn init<R: Runtime>(callback: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app, _api| {
            #[allow(unused_mut)]
            let mut id = app.config().identifier.clone();
            #[cfg(feature = "semver")]
            {
                id.push('_');
                id.push_str(semver_compat_string(&app.package_info().version).as_str());
            }

            let class_name = encode_wide(format!("{id}-sic"));
            let window_name = encode_wide(format!("{id}-siw"));
            let mutex_name = encode_wide(format!("{id}-sim"));

            let hmutex =
                unsafe { CreateMutexW(std::ptr::null(), true.into(), mutex_name.as_ptr()) };

            if already_running(hmutex, unsafe { GetLastError() }) {
                unsafe {
                    let hwnd = FindWindowW(class_name.as_ptr(), window_name.as_ptr());

                    if !hwnd.is_null() {
                        // Windows lets us bring a window to the front, but not the first
                        // instance. Hand that right over before we exit, so focusing a window
                        // from the callback works. Windows takes it back if the user switches
                        // to another app in the meantime.
                        let mut pid = 0;
                        GetWindowThreadProcessId(hwnd, &mut pid);
                        if pid != 0 {
                            AllowSetForegroundWindow(pid);
                        }

                        let cwd = std::env::current_dir().unwrap_or_default();
                        let cwd = cwd.to_str().unwrap_or_default();

                        let args = std::env::args().collect::<Vec<String>>().join("|");

                        let data = format!("{cwd}|{args}\0",);

                        let bytes = data.as_bytes();
                        let cds = COPYDATASTRUCT {
                            dwData: WMCOPYDATA_SINGLE_INSTANCE_DATA,
                            cbData: bytes.len() as _,
                            lpData: bytes.as_ptr() as _,
                        };

                        SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as _);

                        app.cleanup_before_exit();
                        std::process::exit(0);
                    }
                }
            } else {
                app.manage(MutexHandle(hmutex as _));

                let userdata = UserData {
                    app: app.clone(),
                    callback,
                };
                let userdata = Box::into_raw(Box::new(userdata));
                let hwnd = create_event_target_window::<R>(&class_name, &window_name, userdata);
                app.manage(TargetWindowHandle(hwnd as _));
            }

            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(hmutex) = manager.try_state::<MutexHandle>() {
        unsafe {
            ReleaseMutex(hmutex.0 as _);
            CloseHandle(hmutex.0 as _);
        }
    }
    if let Some(hwnd) = manager.try_state::<TargetWindowHandle>() {
        unsafe { DestroyWindow(hwnd.0 as _) };
    }
}

/// Whether the `CreateMutexW` call that returned `hmutex`, leaving `error` as the last
/// error, found the mutex of an instance that is already running.
///
/// An instance running at a higher integrity level, such as an elevated one, holds a mutex
/// this process may not open with the access `CreateMutexW` requests, so the call fails
/// with `ERROR_ACCESS_DENIED` instead of returning it. A new name cannot be refused, since
/// every process in a session may create objects in the session's namespace, so that
/// failure means the mutex exists.
fn already_running(hmutex: HANDLE, error: WIN32_ERROR) -> bool {
    if hmutex.is_null() {
        error == ERROR_ACCESS_DENIED
    } else {
        error == ERROR_ALREADY_EXISTS
    }
}

unsafe extern "system" fn single_instance_window_proc<R: Runtime>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam as *const CREATESTRUCTW);
            let userdata = create_struct.lpCreateParams as *const UserData<R>;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, userdata as _);
            0
        }

        WM_COPYDATA => {
            let cds_ptr = lparam as *const COPYDATASTRUCT;
            if (*cds_ptr).dwData == WMCOPYDATA_SINGLE_INSTANCE_DATA {
                let userdata = UserData::<R>::from_hwnd(hwnd);

                let data = CStr::from_ptr((*cds_ptr).lpData as _).to_string_lossy();
                let mut s = data.split('|');
                let cwd = s.next().unwrap();
                let args = s.map(|s| s.to_string()).collect();

                userdata.run_callback(args, cwd.to_string());
            }
            1
        }

        WM_DESTROY => {
            let userdata = UserData::<R>::from_hwnd_raw(hwnd);
            drop(Box::from_raw(userdata));
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_event_target_window<R: Runtime>(
    class_name: &[u16],
    window_name: &[u16],
    userdata: *const UserData<R>,
) -> HWND {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(single_instance_window_proc::<R>),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        RegisterClassExW(&class);

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE
            | WS_EX_TRANSPARENT
            | WS_EX_LAYERED
            // WS_EX_TOOLWINDOW prevents this window from ever showing up in the taskbar, which
            // we want to avoid. If you remove this style, this window won't show up in the
            // taskbar *initially*, but it can show up at some later point. This can sometimes
            // happen on its own after several hours have passed, although this has proven
            // difficult to reproduce. Alternatively, it can be manually triggered by killing
            // `explorer.exe` and then starting the process back up.
            // It is unclear why the bug is triggered by waiting for several hours.
            | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            userdata as _,
        );
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            // The window technically has to be visible to receive WM_PAINT messages (which are used
            // for delivering events during resizes), but it isn't displayed to the user because of
            // the LAYERED style.
            (WS_VISIBLE | WS_POPUP) as isize,
        );
        hwnd
    }
}

pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongW(hwnd, index, value as _) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongPtrW(hwnd, index, value)
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongW(hwnd, index) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongPtrW(hwnd, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Security::{
        ACL, ACL_REVISION, InitializeAcl, InitializeSecurityDescriptor, PSECURITY_DESCRIPTOR,
        SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR, SetSecurityDescriptorDacl,
    };

    // Generated by windows-sys under `Win32_System_SystemServices`, which the crate does not
    // otherwise need.
    const SECURITY_DESCRIPTOR_REVISION: u32 = 1;

    fn mutex_name(purpose: &str) -> Vec<u16> {
        encode_wide(format!(
            "tauri-plugin-single-instance-test-{purpose}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn a_created_or_opened_mutex_answers_by_its_error() {
        let name = mutex_name("open");
        unsafe {
            let first = CreateMutexW(std::ptr::null(), true.into(), name.as_ptr());
            assert!(!already_running(first, GetLastError()));

            let second = CreateMutexW(std::ptr::null(), true.into(), name.as_ptr());
            assert!(already_running(second, GetLastError()));

            CloseHandle(second);
            CloseHandle(first);
        }
    }

    // An empty DACL refuses this process the access `CreateMutexW` requests, as the mutex of
    // an elevated instance refuses a process that is not elevated.
    #[test]
    fn a_mutex_this_process_may_not_open_is_a_running_instance() {
        let name = mutex_name("refused");
        unsafe {
            let mut acl: ACL = std::mem::zeroed();
            assert_ne!(
                InitializeAcl(&mut acl, size_of::<ACL>() as u32, ACL_REVISION),
                0
            );
            let mut descriptor: SECURITY_DESCRIPTOR = std::mem::zeroed();
            let descriptor: PSECURITY_DESCRIPTOR = (&raw mut descriptor).cast();
            assert_ne!(
                InitializeSecurityDescriptor(descriptor, SECURITY_DESCRIPTOR_REVISION),
                0
            );
            assert_ne!(
                SetSecurityDescriptorDacl(descriptor, true.into(), &acl, false.into()),
                0
            );
            let attributes = SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: false.into(),
            };
            let held = CreateMutexW(&attributes, false.into(), name.as_ptr());
            assert!(!held.is_null());

            let refused = CreateMutexW(std::ptr::null(), true.into(), name.as_ptr());
            let error = GetLastError();
            assert!(refused.is_null());
            assert_eq!(error, ERROR_ACCESS_DENIED);
            assert!(already_running(refused, error));

            CloseHandle(held);
        }
    }
}
