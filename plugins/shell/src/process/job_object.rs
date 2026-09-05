// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Job object backing `process_group` on Windows, adapted from
//! [process-wrap](https://github.com/watchexec/process-wrap) (Apache-2.0 OR MIT).

use std::{io, mem, os::windows::io::AsRawHandle, process::Child, ptr};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        JobObjects::{AssignProcessToJobObject, CreateJobObjectW, TerminateJobObject},
        Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
    },
};

/// Owned job object handle. Closing it leaves the processes in the job running.
pub(crate) struct JobObject(HANDLE);

// SAFETY: the handle is only ever passed to thread-safe Win32 calls.
unsafe impl Send for JobObject {}
unsafe impl Sync for JobObject {}

impl JobObject {
    /// Creates a job object, assigns the `CREATE_SUSPENDED` `child` to it and resumes the child.
    pub(crate) fn assign(child: &Child) -> io::Result<Self> {
        let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = Self(job);
        if unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        resume_threads(child.id())?;
        Ok(job)
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        if unsafe { TerminateJobObject(self.0, 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for JobObject {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

/// `std` closes the main thread handle right after `CreateProcess`, so a suspended
/// child can only be resumed by looking its threads up again.
fn resume_threads(pid: u32) -> io::Result<()> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let result = resume_snapshot_threads(snapshot, pid);
    unsafe { CloseHandle(snapshot) };
    result
}

fn resume_snapshot_threads(snapshot: HANDLE, pid: u32) -> io::Result<()> {
    let mut entry: THREADENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
    let mut found = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while found {
        if entry.th32OwnerProcessID == pid {
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if thread.is_null() {
                return Err(io::Error::last_os_error());
            }
            if unsafe { ResumeThread(thread) } == u32::MAX {
                let err = io::Error::last_os_error();
                unsafe { CloseHandle(thread) };
                return Err(err);
            }
            unsafe { CloseHandle(thread) };
        }
        found = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    Ok(())
}
