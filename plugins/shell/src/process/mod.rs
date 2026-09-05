// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    ffi::OsStr,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command as StdCommand, Stdio},
    sync::{Arc, RwLock},
    thread::spawn,
};

#[cfg(unix)]
use std::os::unix::process::{CommandExt, ExitStatusExt};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::{CREATE_NO_WINDOW, CREATE_SUSPENDED};

const NEWLINE_BYTE: u8 = b'\n';

use tauri::async_runtime::{block_on as block_on_task, channel, Receiver, Sender};

pub use encoding_rs::Encoding;
use os_pipe::{pipe, PipeReader, PipeWriter};
use serde::Serialize;
use shared_child::SharedChild;
use tauri::utils::platform;

#[cfg(windows)]
mod job_object;

/// Payload for the [`CommandEvent::Terminated`] command event.
#[derive(Debug, Clone, Serialize)]
pub struct TerminatedPayload {
    /// Exit code of the process.
    pub code: Option<i32>,
    /// If the process was terminated by a signal, represents that signal.
    pub signal: Option<i32>,
}

/// A event sent to the command callback.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum CommandEvent {
    /// If configured for raw output, all bytes written to stderr.
    /// Otherwise, bytes until a newline (\n) or carriage return (\r) is found.
    Stderr(Vec<u8>),
    /// If configured for raw output, all bytes written to stdout.
    /// Otherwise, bytes until a newline (\n) or carriage return (\r) is found.
    Stdout(Vec<u8>),
    /// An error happened waiting for the command to finish or converting the stdout/stderr bytes to a UTF-8 string.
    Error(String),
    /// Command process terminated.
    Terminated(TerminatedPayload),
}

/// The type to spawn commands.
#[derive(Debug)]
pub struct Command {
    cmd: StdCommand,
    raw_out: bool,
    process_group: bool,
}

/// Spawned child process.
pub struct CommandChild {
    inner: Arc<SharedChild>,
    stdin_writer: PipeWriter,
    #[cfg(unix)]
    process_group: bool,
    #[cfg(windows)]
    job: Option<job_object::JobObject>,
}

impl CommandChild {
    /// Writes to process stdin.
    pub fn write(&mut self, buf: &[u8]) -> crate::Result<()> {
        self.stdin_writer.write_all(buf)?;
        Ok(())
    }

    /// Sends a kill signal to the child and waits for it to exit.
    /// With `process_group` enabled this kills the whole process group (POSIX) or job object (Windows).
    pub fn kill(self) -> crate::Result<()> {
        if self.inner.try_wait()?.is_some() {
            return Ok(());
        }

        #[cfg(unix)]
        if self.process_group {
            let pgid = self.inner.id() as libc::pid_t;
            if unsafe { libc::killpg(pgid, libc::SIGKILL) } != 0 {
                let err = std::io::Error::last_os_error();
                // ESRCH: the group emptied out between `try_wait` and `killpg`.
                if err.raw_os_error() != Some(libc::ESRCH) {
                    return Err(err.into());
                }
            }
        } else {
            self.inner.kill()?;
        }

        #[cfg(windows)]
        match &self.job {
            Some(job) => job.terminate()?,
            None => self.inner.kill()?,
        }

        self.inner.wait()?;
        Ok(())
    }

    /// Returns the process pid.
    pub fn pid(&self) -> u32 {
        self.inner.id()
    }
}

/// Describes the result of a process after it has terminated.
#[derive(Debug)]
pub struct ExitStatus {
    // This field is intentionally left private.
    // See: https://github.com/tauri-apps/plugins-workspace/pull/3115.
    code: Option<i32>,
}

impl ExitStatus {
    /// Returns the exit code of the process, if any.
    pub fn code(&self) -> Option<i32> {
        self.code
    }

    /// Returns true if exit status is zero. Signal termination is not considered a success, and success is defined as a zero exit status.
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

/// The output of a finished process.
#[derive(Debug)]
pub struct Output {
    /// The status (exit code) of the process.
    pub status: ExitStatus,
    /// The data that the process wrote to stdout.
    pub stdout: Vec<u8>,
    /// The data that the process wrote to stderr.
    pub stderr: Vec<u8>,
}

fn relative_command_path(command: &Path) -> crate::Result<PathBuf> {
    let exe_path = platform::current_exe()?;

    let exe_dir = exe_path
        .parent()
        .ok_or(crate::Error::CurrentExeHasNoParent)?;

    // If a test is being run, the executable is in the "deps" directory, so we need to go up one level.
    let base_dir = if exe_dir.ends_with("deps") {
        exe_dir.parent().unwrap_or(exe_dir)
    } else {
        exe_dir
    };

    let mut command_path = base_dir.join(command);

    #[cfg(windows)]
    {
        let already_exe = command_path.extension().is_some_and(|ext| ext == "exe");
        if !already_exe {
            // do not use with_extension to retain dots in the command filename
            command_path.as_mut_os_string().push(".exe");
        }
    }

    #[cfg(not(windows))]
    {
        if command_path.extension().is_some_and(|ext| ext == "exe") {
            command_path.set_extension("");
        }
    }

    Ok(command_path)
}

impl From<Command> for StdCommand {
    fn from(cmd: Command) -> StdCommand {
        cmd.cmd
    }
}

impl Command {
    pub(crate) fn new<S: AsRef<OsStr>>(program: S) -> Self {
        log::debug!(
            "Creating sidecar {}",
            program.as_ref().to_str().unwrap_or("")
        );
        let mut command = StdCommand::new(program);

        command.stdout(Stdio::piped());
        command.stdin(Stdio::piped());
        command.stderr(Stdio::piped());
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);

        Self {
            cmd: command,
            raw_out: false,
            process_group: false,
        }
    }

    pub(crate) fn new_sidecar<S: AsRef<Path>>(program: S) -> crate::Result<Self> {
        Ok(Self::new(relative_command_path(program.as_ref())?))
    }

    /// Appends an argument to the command.
    #[must_use]
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg);
        self
    }

    /// Appends arguments to the command.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd.args(args);
        self
    }

    /// Clears the entire environment map for the child process.
    #[must_use]
    pub fn env_clear(mut self) -> Self {
        self.cmd.env_clear();
        self
    }

    /// Inserts or updates an explicit environment variable mapping.
    #[must_use]
    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.cmd.env(key, value);
        self
    }

    /// Adds or updates multiple environment variable mappings.
    #[must_use]
    pub fn envs<I, K, V>(mut self, envs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.cmd.envs(envs);
        self
    }

    /// Sets the working directory for the child process.
    #[must_use]
    pub fn current_dir<P: AsRef<Path>>(mut self, current_dir: P) -> Self {
        self.cmd.current_dir(current_dir);
        self
    }

    /// Configures the reader to output bytes from the child process exactly as received
    pub fn set_raw_out(mut self, raw_out: bool) -> Self {
        self.raw_out = raw_out;
        self
    }

    /// Configures the command to spawn in a new process group (POSIX) or job object (Windows).
    ///
    /// When enabled, killing the child process will also kill all processes in the group,
    /// which is useful for programs that spawn child processes (e.g. PyInstaller wrappers).
    #[must_use]
    pub fn set_process_group(mut self, process_group: bool) -> Self {
        self.process_group = process_group;
        self
    }

    /// Spawns the command.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_shell::{process::CommandEvent, ShellExt};
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle().clone();
    ///     tauri::async_runtime::spawn(async move {
    ///       let (mut rx, mut child) = handle
    ///         .shell()
    ///         .command("cargo")
    ///         .args(["tauri", "dev"])
    ///         .spawn()
    ///         .expect("Failed to spawn cargo");
    ///
    ///       let mut i = 0;
    ///       while let Some(event) = rx.recv().await {
    ///         if let CommandEvent::Stdout(line) = event {
    ///           println!("got: {}", String::from_utf8(line).unwrap());
    ///           i += 1;
    ///           if i == 4 {
    ///             child.write("message from Rust\n".as_bytes()).unwrap();
    ///             i = 0;
    ///           }
    ///         }
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    ///
    /// Depending on the command you spawn, it might output in a specific encoding, to parse the output lines in this case:
    ///
    /// ```rust,no_run
    /// use tauri_plugin_shell::{process::{CommandEvent, Encoding}, ShellExt};
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle().clone();
    ///     tauri::async_runtime::spawn(async move {
    ///       let (mut rx, mut child) = handle
    ///         .shell()
    ///         .command("some-program")
    ///         .arg("some-arg")
    ///         .spawn()
    ///         .expect("Failed to spawn some-program");
    ///
    ///       let encoding = Encoding::for_label(b"windows-1252").unwrap();
    ///       while let Some(event) = rx.recv().await {
    ///         if let CommandEvent::Stdout(line) = event {
    ///           let (decoded, _, _) = encoding.decode(&line);
    ///           println!("got: {decoded}");
    ///         }
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    pub fn spawn(self) -> crate::Result<(Receiver<CommandEvent>, CommandChild)> {
        let raw = self.raw_out;
        let process_group = self.process_group;
        let mut command: StdCommand = self.into();
        let (stdout_reader, stdout_writer) = pipe()?;
        let (stderr_reader, stderr_writer) = pipe()?;
        let (stdin_reader, stdin_writer) = pipe()?;
        command.stdout(stdout_writer);
        command.stderr(stderr_writer);
        command.stdin(stdin_reader);

        #[cfg(unix)]
        if process_group {
            command.process_group(0);
        }
        #[cfg(windows)]
        if process_group {
            // Start suspended so nothing can be spawned before the child is in the job.
            command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
        }

        let child = command.spawn()?;

        #[cfg(windows)]
        let job = if process_group {
            match job_object::JobObject::assign(&child) {
                Ok(job) => Some(job),
                Err(e) => {
                    // Don't leave the suspended child behind.
                    let mut child = child;
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(e.into());
                }
            }
        } else {
            None
        };

        let child = Arc::new(SharedChild::new(child)?);
        let child_ = child.clone();
        let guard = Arc::new(RwLock::new(()));

        let (tx, rx) = channel(1);

        spawn_pipe_reader(
            tx.clone(),
            guard.clone(),
            stdout_reader,
            CommandEvent::Stdout,
            raw,
        );
        spawn_pipe_reader(
            tx.clone(),
            guard.clone(),
            stderr_reader,
            CommandEvent::Stderr,
            raw,
        );

        spawn(move || {
            let _ = match child_.wait() {
                Ok(status) => {
                    let _l = guard.write().unwrap();
                    block_on_task(async move {
                        tx.send(CommandEvent::Terminated(TerminatedPayload {
                            code: status.code(),
                            #[cfg(windows)]
                            signal: None,
                            #[cfg(unix)]
                            signal: status.signal(),
                        }))
                        .await
                    })
                }
                Err(e) => {
                    let _l = guard.write().unwrap();
                    block_on_task(async move { tx.send(CommandEvent::Error(e.to_string())).await })
                }
            };
        });

        Ok((
            rx,
            CommandChild {
                inner: child,
                stdin_writer,
                #[cfg(unix)]
                process_group,
                #[cfg(windows)]
                job,
            },
        ))
    }

    /// Executes a command as a child process, waiting for it to finish and collecting its exit status.
    /// Stdin, stdout and stderr are ignored.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use tauri_plugin_shell::ShellExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let status = tauri::async_runtime::block_on(async move { app.shell().command("which").args(["ls"]).status().await.unwrap() });
    ///     println!("`which` finished with status: {:?}", status.code());
    ///     Ok(())
    ///   });
    /// ```
    pub async fn status(self) -> crate::Result<ExitStatus> {
        let (mut rx, _child) = self.spawn()?;
        let mut code = None;
        #[allow(clippy::collapsible_match)]
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Terminated(payload) = event {
                code = payload.code;
            }
        }
        Ok(ExitStatus { code })
    }

    /// Executes the command as a child process, waiting for it to finish and collecting all of its output.
    /// Stdin is ignored.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_shell::ShellExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let output = tauri::async_runtime::block_on(async move { app.shell().command("echo").args(["TAURI"]).output().await.unwrap() });
    ///     assert!(output.status.success());
    ///     assert_eq!(String::from_utf8(output.stdout).unwrap(), "TAURI");
    ///     Ok(())
    ///   });
    /// ```
    pub async fn output(self) -> crate::Result<Output> {
        let (mut rx, _child) = self.spawn()?;

        let mut code = None;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Terminated(payload) => {
                    code = payload.code;
                }
                CommandEvent::Stdout(line) => {
                    stdout.extend(line);
                    stdout.push(NEWLINE_BYTE);
                }
                CommandEvent::Stderr(line) => {
                    stderr.extend(line);
                    stderr.push(NEWLINE_BYTE);
                }
                CommandEvent::Error(_) => {}
            }
        }
        Ok(Output {
            status: ExitStatus { code },
            stdout,
            stderr,
        })
    }
}

fn read_raw_bytes<F: Fn(Vec<u8>) -> CommandEvent + Send + Copy + 'static>(
    mut reader: BufReader<PipeReader>,
    tx: Sender<CommandEvent>,
    wrapper: F,
) {
    loop {
        let result = reader.fill_buf();
        match result {
            Ok(buf) => {
                let length = buf.len();
                if length == 0 {
                    break;
                }
                let tx_ = tx.clone();
                let _ = block_on_task(async move { tx_.send(wrapper(buf.to_vec())).await });
                reader.consume(length);
            }
            Err(e) => {
                let tx_ = tx.clone();
                let _ = block_on_task(
                    async move { tx_.send(CommandEvent::Error(e.to_string())).await },
                );
            }
        }
    }
}

fn read_line<F: Fn(Vec<u8>) -> CommandEvent + Send + Copy + 'static>(
    mut reader: BufReader<PipeReader>,
    tx: Sender<CommandEvent>,
    wrapper: F,
) {
    loop {
        let mut buf = Vec::new();
        match tauri::utils::io::read_line(&mut reader, &mut buf) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                let tx_ = tx.clone();
                let _ = block_on_task(async move { tx_.send(wrapper(buf)).await });
            }
            Err(e) => {
                let _ =
                    block_on_task(async move { tx.send(CommandEvent::Error(e.to_string())).await });
                break;
            }
        }
    }
}

fn spawn_pipe_reader<F: Fn(Vec<u8>) -> CommandEvent + Send + Copy + 'static>(
    tx: Sender<CommandEvent>,
    guard: Arc<RwLock<()>>,
    pipe_reader: PipeReader,
    wrapper: F,
    raw_out: bool,
) {
    spawn(move || {
        let _lock = guard.read().unwrap();
        let reader = BufReader::new(pipe_reader);

        if raw_out {
            read_raw_bytes(reader, tx, wrapper);
        } else {
            read_line(reader, tx, wrapper);
        }
    });
}

// tests for the commands functions.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_command_path_resolves() {
        let cwd_parent = platform::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent() // Go up once more to get out of the "deps" directory
            .unwrap()
            .to_owned();
        assert_eq!(
            relative_command_path(Path::new("Tauri.Example")).unwrap(),
            cwd_parent.join(if cfg!(windows) {
                "Tauri.Example.exe"
            } else {
                "Tauri.Example"
            })
        );
        assert_eq!(
            relative_command_path(Path::new("Tauri.Example.exe")).unwrap(),
            cwd_parent.join(if cfg!(windows) {
                "Tauri.Example.exe"
            } else {
                "Tauri.Example"
            })
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_spawn_output() {
        let cmd = Command::new("cat").args(["test/test.txt"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        assert_eq!(String::from_utf8(line).unwrap(), "This is a test doc!");
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_spawn_raw_output() {
        let cmd = Command::new("cat").args(["test/test.txt"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        assert_eq!(String::from_utf8(line).unwrap(), "This is a test doc!");
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(windows))]
    #[test]
    // test the failure case
    fn test_cmd_spawn_fail() {
        let cmd = Command::new("cat").args(["test/"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(1));
                    }
                    CommandEvent::Stderr(line) => {
                        assert_eq!(
                            String::from_utf8(line).unwrap(),
                            "cat: test/: Is a directory\n"
                        );
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(windows))]
    #[test]
    // test the failure case (raw encoding)
    fn test_cmd_spawn_raw_fail() {
        let cmd = Command::new("cat").args(["test/"]);
        let (mut rx, _) = cmd.spawn().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(1));
                    }
                    CommandEvent::Stderr(line) => {
                        assert_eq!(
                            String::from_utf8(line).unwrap(),
                            "cat: test/: Is a directory\n"
                        );
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_output_output() {
        let cmd = Command::new("cat").args(["test/test.txt"]);
        let output = tauri::async_runtime::block_on(cmd.output()).unwrap();

        assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "This is a test doc!\n"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_output_output_fail() {
        let cmd = Command::new("cat").args(["test/"]);
        let output = tauri::async_runtime::block_on(cmd.output()).unwrap();

        assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "cat: test/: Is a directory\n\n"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_spawn_process_group_output() {
        let cmd = Command::new("cat")
            .args(["test/test.txt"])
            .set_process_group(true);
        let (mut rx, _) = cmd.spawn().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(payload) => {
                        assert_eq!(payload.code, Some(0));
                    }
                    CommandEvent::Stdout(line) => {
                        assert_eq!(String::from_utf8(line).unwrap(), "This is a test doc!");
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(not(windows))]
    #[test]
    fn test_cmd_process_group_kill() {
        // Spawn a shell that runs a sleep command as a child process.
        // With process_group enabled, killing the parent should also kill the child.
        let cmd = Command::new("sh")
            .args(["-c", "sleep 60"])
            .set_process_group(true);
        let (mut rx, child) = cmd.spawn().unwrap();
        let pid = child.pid();

        // Verify the process is running
        let ret = unsafe { libc::kill(pid as i32, 0) };
        assert_eq!(ret, 0, "process should be running");

        // Kill the process group
        child.kill().unwrap();

        tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Terminated(payload) = event {
                    // Process was killed by signal, so code is None and signal is Some
                    assert!(payload.code.is_none() || payload.signal.is_some());
                    break;
                }
            }
        });

        // Verify the process group is gone
        let ret = unsafe { libc::killpg(pid as i32, 0) };
        assert_ne!(ret, 0, "process group should no longer exist");
    }

    #[test]
    fn test_cmd_process_group_output() {
        // Hangs on Windows if the suspended child is never resumed.
        #[cfg(not(windows))]
        let cmd = Command::new("cat").args(["test/test.txt"]);
        #[cfg(windows)]
        let cmd = Command::new("cmd").args(["/C", "type test\\test.txt"]);

        let output = tauri::async_runtime::block_on(cmd.set_process_group(true).output()).unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            "This is a test doc!"
        );
    }

    #[test]
    fn test_cmd_kill_after_exit() {
        for process_group in [false, true] {
            #[cfg(not(windows))]
            let cmd = Command::new("true");
            #[cfg(windows)]
            let cmd = Command::new("cmd").args(["/C", "exit 0"]);
            let (rx, child) = cmd.set_process_group(process_group).spawn().unwrap();
            wait_for_terminated(rx);
            child.kill().unwrap();
        }
    }

    /// The PyInstaller-style wrapper script for the current platform: spawns
    /// a long-running grandchild, prints its PID, then waits on it.
    fn sim_command() -> Command {
        if cfg!(windows) {
            Command::new("powershell").args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "test/pyinstaller_sim.ps1",
            ])
        } else {
            Command::new("sh").args(["test/pyinstaller_sim.sh"])
        }
    }

    /// Reads command output until the wrapper script reports the grandchild pid.
    fn read_grandchild_pid(rx: &mut Receiver<CommandEvent>) -> u32 {
        tauri::async_runtime::block_on(async {
            let mut pid = None;
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Stdout(line) = &event {
                    let line_str = String::from_utf8_lossy(line);
                    if let Some(rest) = line_str.strip_prefix("CHILD_PID=") {
                        pid = rest.trim().parse::<u32>().ok();
                    }
                }
                if pid.is_some() {
                    break;
                }
            }
            pid.expect("should have received CHILD_PID from script")
        })
    }

    /// Asserts that the child produces a `Terminated` event.
    fn wait_for_terminated(mut rx: Receiver<CommandEvent>) {
        let got = tauri::async_runtime::block_on(async move {
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Terminated(_) = event {
                    return true;
                }
            }
            false
        });
        assert!(got, "expected a Terminated event");
    }

    #[cfg(not(windows))]
    fn pid_alive(pid: u32) -> bool {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }

    #[cfg(windows)]
    fn pid_alive(pid: u32) -> bool {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, STILL_ACTIVE},
            System::Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            },
        };
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return false;
        }
        let mut code = 0u32;
        let alive =
            unsafe { GetExitCodeProcess(handle, &mut code) } != 0 && code == STILL_ACTIVE as u32;
        unsafe { CloseHandle(handle) };
        alive
    }

    #[cfg(not(windows))]
    fn force_kill(pid: u32) {
        unsafe { libc::kill(pid as i32, libc::SIGKILL) };
    }

    #[cfg(windows)]
    fn force_kill(pid: u32) {
        use windows_sys::Win32::{
            Foundation::CloseHandle,
            System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE},
        };
        let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
        if !handle.is_null() {
            unsafe { TerminateProcess(handle, 1) };
            unsafe { CloseHandle(handle) };
        }
    }

    /// End-to-end test simulating the PyInstaller scenario from issue #1332.
    ///
    /// PyInstaller wraps the real application in a thin bootloader process.
    /// Without process groups, killing the bootloader orphans the real app.
    /// This test verifies that with `process_group` enabled, killing the
    /// wrapper also kills the grandchild process.
    #[test]
    fn test_pyinstaller_simulation_without_process_group() {
        // Without process_group: killing the wrapper does NOT kill the grandchild.
        let (mut rx, child) = sim_command().spawn().unwrap();

        let grandchild_pid = read_grandchild_pid(&mut rx);
        assert!(
            pid_alive(grandchild_pid),
            "grandchild should be running before kill"
        );

        // Kill just the direct child (no process group)
        child.kill().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // The grandchild is STILL alive — this is the bug
        assert!(
            pid_alive(grandchild_pid),
            "grandchild should survive when process_group is off"
        );

        // Clean up the orphaned grandchild. This also closes the inherited
        // stdout pipe, which the Terminated event is gated on.
        force_kill(grandchild_pid);
        wait_for_terminated(rx);
    }

    #[test]
    fn test_pyinstaller_simulation_with_process_group() {
        // With process_group: killing the wrapper ALSO kills the grandchild.
        let (mut rx, child) = sim_command().set_process_group(true).spawn().unwrap();

        let grandchild_pid = read_grandchild_pid(&mut rx);
        assert!(
            pid_alive(grandchild_pid),
            "grandchild should be running before kill"
        );

        // Kill the process group
        child.kill().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // The grandchild should now be DEAD
        assert!(
            !pid_alive(grandchild_pid),
            "grandchild should be killed when process_group is on"
        );

        wait_for_terminated(rx);
    }
}
