// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::SingleInstanceCallback;
use std::ffi::CStr;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, RunEvent, Runtime,
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WAIT_ABANDONED,
        WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM,
    },
    System::{
        DataExchange::COPYDATASTRUCT,
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
    },
    UI::WindowsAndMessaging::{
        self as w32wm, CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW,
        RegisterClassExW, SendMessageW, CREATESTRUCTW, GWLP_USERDATA, GWL_STYLE,
        WINDOW_LONG_PTR_INDEX, WM_COPYDATA, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
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

            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                // Another instance holds the mutex, but its event target window may not
                // exist yet: the first instance creates the mutex before the window, and a
                // second launch inside that gap used to see the mutex without a window and
                // silently continue as an unguarded extra instance (the gap is normally
                // microseconds, but heavy system load stretches it to hundreds of
                // milliseconds). Poll for the window while waiting on the mutex, so this
                // process either forwards its args to the primary once the window is up,
                // or inherits the primary role when the holder releases the mutex or dies
                // (WAIT_ABANDONED — e.g. an updater respawning the app while the old
                // process is still exiting).
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    unsafe {
                        let hwnd = FindWindowW(class_name.as_ptr(), window_name.as_ptr());

                        if !hwnd.is_null() {
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

                    if std::time::Instant::now() >= deadline {
                        // No primary window appeared and nobody released the mutex
                        // (e.g. the holder is wedged before creating its window).
                        // Fail open rather than block startup forever.
                        break;
                    }

                    match unsafe { WaitForSingleObject(hmutex, 50) } {
                        // Ownership acquired: the previous holder released the mutex or
                        // died holding it. Proceed as the primary instance.
                        WAIT_OBJECT_0 | WAIT_ABANDONED => break,
                        WAIT_TIMEOUT => {}
                        // WAIT_FAILED: fail open.
                        _ => break,
                    }
                }
            }

            // Reached as the primary instance (mutex created new, or ownership inherited
            // from a dying holder) or via the fail-open paths above. Register the event
            // target window in all cases so later launches can forward to this process;
            // previously a fail-open instance created no window and leaked the mutex
            // handle silently.
            app.manage(MutexHandle(hmutex as _));

            let userdata = UserData {
                app: app.clone(),
                callback,
            };
            let userdata = Box::into_raw(Box::new(userdata));
            let hwnd = create_event_target_window::<R>(&class_name, &window_name, userdata);
            app.manage(TargetWindowHandle(hwnd as _));

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
