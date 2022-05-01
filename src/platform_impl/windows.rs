#![cfg(target_os = "windows")]

use std::{cell::RefCell, ffi::CStr, rc::Rc};

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    Manager, RunEvent, Runtime,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
    System::{
        DataExchange::COPYDATASTRUCT,
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateMutexW, ReleaseMutex},
    },
    UI::WindowsAndMessaging::{
        self as w32wm, CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW,
        RegisterClassExW, SendMessageW, GWL_STYLE, GWL_USERDATA, WINDOW_LONG_PTR_INDEX,
        WM_COPYDATA, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT, WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
    },
};

pub fn init<R: Runtime>(f: Box<SingleInstanceCallback>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app| {
            let hmutex = unsafe {
                CreateMutexW(
                    std::ptr::null(),
                    true.into(),
                    encode_wide("tauri-plugin-single-instance-mutex").as_ptr(),
                )
            };

            let app_name = &app.package_info().name;
            let class_name = format!("{}-single-instance-class", app_name);
            let window_name = format!("{}-single-instance-window", app_name);

            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                unsafe {
                    let hwnd = FindWindowW(
                        encode_wide(&class_name).as_ptr(),
                        encode_wide(&window_name).as_ptr(),
                    );

                    if hwnd != 0 {
                        let data = format!(
                            "{}|{}\0",
                            std::env::current_dir()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default(),
                            std::env::args().collect::<Vec<String>>().join("|")
                        );
                        let bytes = data.as_bytes();
                        let cds = COPYDATASTRUCT {
                            dwData: WMCOPYDATA_SINGLE_INSTANCE_DATA,
                            cbData: bytes.len() as _,
                            lpData: bytes.as_ptr() as _,
                        };
                        let ret = SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as _);
                        if ret == CLOSE_NEW_INSTANCE {
                            std::process::exit(0);
                        }
                    }
                }
            }

            app.manage(MutexHandle(hmutex));

            unsafe {
                let class = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: 0,
                    lpfnWndProc: Some(single_instance_window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: GetModuleHandleW(std::ptr::null()),
                    hIcon: 0,
                    hCursor: 0,
                    hbrBackground: 0,
                    lpszMenuName: std::ptr::null(),
                    lpszClassName: encode_wide(&class_name).as_ptr(),
                    hIconSm: 0,
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
                    encode_wide(&class_name).as_ptr(),
                    encode_wide(&window_name).as_ptr(),
                    WS_OVERLAPPED,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null(),
                );
                SetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                    // The window technically has to be visible to receive WM_PAINT messages (which are used
                    // for delivering events during resizes), but it isn't displayed to the user because of
                    // the LAYERED style.
                    (WS_VISIBLE | WS_POPUP) as isize,
                );

                SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(Box::new(f)) as _);

                app.manage(TargetWindowHandle(hwnd));
            }
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                let hmutex = app.state::<MutexHandle>().0;
                unsafe {
                    ReleaseMutex(hmutex);
                    CloseHandle(hmutex);
                };

                let hwnd = app.state::<TargetWindowHandle>().0;
                unsafe { DestroyWindow(hwnd) };
            }
        })
        .build()
}

unsafe extern "system" fn single_instance_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let callback_ptr = GetWindowLongPtrW(hwnd, GWL_USERDATA) as *mut Box<SingleInstanceCallback>;

    match msg {
        WM_COPYDATA => {
            let ret = Rc::new(RefCell::new(1));

            let cds_ptr = lparam as *const COPYDATASTRUCT;
            if (*cds_ptr).dwData == WMCOPYDATA_SINGLE_INSTANCE_DATA {
                let data = CStr::from_ptr((*cds_ptr).lpData as _).to_string_lossy();
                let mut s = data.split("|");
                let cwd = s.next().unwrap();
                let args = s.into_iter().map(|s| s.to_string()).collect();
                let ret_c = Rc::clone(&ret);
                (*callback_ptr)(
                    args,
                    cwd.to_string(),
                    Box::new(move || {
                        let mut ret = ret_c.borrow_mut();
                        *ret = CLOSE_NEW_INSTANCE;
                    }),
                );
            }
            ret.take()
        }

        WM_DESTROY => {
            let _ = Box::from_raw(callback_ptr);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

struct MutexHandle(isize);
struct TargetWindowHandle(isize);
const WMCOPYDATA_SINGLE_INSTANCE_DATA: usize = 1542;
const CLOSE_NEW_INSTANCE: isize = 1542;

pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    unsafe { w32wm::SetWindowLongW(hwnd, index, value as _) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    unsafe { w32wm::SetWindowLongPtrW(hwnd, index, value) }
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    unsafe { w32wm::GetWindowLongW(hwnd, index) as _ }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
pub fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    unsafe { w32wm::GetWindowLongPtrW(hwnd, index) }
}
