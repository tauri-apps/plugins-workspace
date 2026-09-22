// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  platform
} from '../helpers/index.js'

// The example's shell scope allows `sh -c <script>` and `cmd /C <script>`.
const shell =
  platform === 'win32'
    ? { program: 'cmd', flag: '/C' }
    : { program: 'sh', flag: '-c' }

// Running a child process works on desktop and on Android (`/system/bin/sh`),
// but iOS does not let an app spawn one at all. The scope specs below stay on
// every platform: `prepare_cmd` rejects before anything is executed.
const itSpawns = platform === 'ios' ? it.skip : it

describePlugin('shell', () => {
  itSpawns('execute collects stdout, stderr and the exit code', async () => {
    const output = await tauri(
      async (api, program, flag, script) => {
        const result = await api.shell.Command.create(program, [
          flag,
          script
        ]).execute()
        return {
          code: result.code,
          signal: result.signal,
          stdout: result.stdout.trim(),
          stderr: result.stderr.trim()
        }
      },
      shell.program,
      shell.flag,
      platform === 'win32'
        ? 'echo hello from e2e && echo warning 1>&2 && exit 3'
        : 'echo "hello from e2e"; echo "warning" >&2; exit 3'
    )
    expect(output.code).toBe(3)
    expect(output.signal).toBeNull()
    expect(output.stdout).toBe('hello from e2e')
    expect(output.stderr).toBe('warning')
  })

  itSpawns(
    'execute passes environment variables and the working directory',
    async () => {
      const cwd = await tauri((api) => api.path.appDataDir())
      const output = await tauri(
        async (api, program, flag, script, cwd) => {
          const result = await api.shell.Command.create(
            program,
            [flag, script],
            {
              cwd,
              env: { E2E_VALUE: 'from-e2e' }
            }
          ).execute()
          return { code: result.code, stdout: result.stdout.trim() }
        },
        shell.program,
        shell.flag,
        platform === 'win32'
          ? 'echo %E2E_VALUE% && cd'
          : 'echo "$E2E_VALUE"; pwd',
        cwd
      )
      expect(output.code).toBe(0)
      const [value, reportedCwd] = output.stdout.split(/\r?\n/)
      expect(value).toBe('from-e2e')
      // the shell may print the directory with a different path style
      expect(reportedCwd?.replace(/\\/g, '/').toLowerCase()).toBe(
        cwd
          .replace(/[\\/]+$/, '')
          .replace(/\\/g, '/')
          .toLowerCase()
      )
    }
  )

  itSpawns(
    'spawn streams stdout and stderr lines and reports close',
    async () => {
      const events = await tauri(
        (api, program, flag, script) =>
          new Promise<{
            stdout: string[]
            stderr: string[]
            close: { code: number | null; signal: number | null }
            pid: number
          }>((resolve, reject) => {
            const stdout: string[] = []
            const stderr: string[] = []
            let pid = 0
            const command = api.shell.Command.create(program, [flag, script])
            command.stdout.on('data', (line) => stdout.push(line.trim()))
            command.stderr.on('data', (line) => stderr.push(line.trim()))
            command.on('error', (error) => reject(new Error(error)))
            command.on('close', (payload) =>
              resolve({
                stdout,
                stderr,
                close: { code: payload.code, signal: payload.signal },
                pid
              })
            )
            command
              .spawn()
              .then((child) => {
                pid = child.pid
              })
              .catch(reject)
            setTimeout(() => reject(new Error('command never closed')), 15000)
          }),
        shell.program,
        shell.flag,
        platform === 'win32'
          ? 'echo one && echo two && echo err 1>&2'
          : 'echo one; echo two; echo err >&2'
      )
      expect(events.pid).toBeGreaterThan(0)
      expect(events.stdout).toEqual(['one', 'two'])
      expect(events.stderr).toEqual(['err'])
      expect(events.close.code).toBe(0)
    }
  )

  itSpawns('write sends to stdin', async () => {
    const output = await tauri(
      (api, program, flag, script) =>
        new Promise<string>((resolve, reject) => {
          let out = ''
          const command = api.shell.Command.create(program, [flag, script])
          command.stdout.on('data', (line) => {
            out += line
          })
          command.on('error', (error) => reject(new Error(error)))
          command.on('close', () => resolve(out.trim()))
          command
            .spawn()
            .then((child) => child.write('ping from e2e\n'))
            .catch(reject)
          setTimeout(() => reject(new Error('command never closed')), 15000)
        }),
      shell.program,
      shell.flag,
      // `call` makes cmd expand `%line%` after `set /p` ran, rather than
      // when the line is parsed
      platform === 'win32'
        ? 'set /p line= & call echo got: %line%'
        : 'read line; echo "got: $line"'
    )
    expect(output).toBe('got: ping from e2e')
  })

  itSpawns('kill terminates a running child', async () => {
    const result = await tauri(
      (api, program, flag, script) =>
        new Promise<{ code: number | null; signal: number | null }>(
          (resolve, reject) => {
            const command = api.shell.Command.create(program, [flag, script])
            command.on('error', (error) => reject(new Error(error)))
            command.on('close', (payload) =>
              resolve({ code: payload.code, signal: payload.signal })
            )
            command
              .spawn()
              .then((child) =>
                // give the shell a moment to start before killing it
                new Promise((r) => setTimeout(r, 500)).then(() => child.kill())
              )
              .catch(reject)
            setTimeout(() => reject(new Error('child was not killed')), 15000)
          }
        ),
      shell.program,
      shell.flag,
      platform === 'win32' ? 'ping -n 30 127.0.0.1 > NUL' : 'sleep 30'
    )
    if (platform === 'win32') {
      // TerminateProcess sets an exit code of 1
      expect(result.code).not.toBe(0)
    } else {
      // killed by SIGKILL, so no exit code
      expect(result.code).toBeNull()
      expect(result.signal).toBe(9)
    }
  })

  it('rejects programs that are not in the scope', async () => {
    const message = await tauriError((api) =>
      api.shell.Command.create('e2e-not-allowed').execute()
    )
    expect(message).toMatch(/program not allowed on the configured shell scope/)
  })

  it('rejects arguments that do not match the scope', async () => {
    // the scope only allows `-c`/`/C` followed by a non-empty script
    const message = await tauriError(
      (api, program) =>
        api.shell.Command.create(program, ['--version']).execute(),
      shell.program
    )
    expect(message).toMatch(/not allowed|validator|scope/i)
  })

  it('open rejects URLs outside the default scope', async () => {
    // `shell:default` only allows http(s), mailto and tel URLs
    const message = await tauriError((api) =>
      api.shell.open('ftp://example.com')
    )
    expect(message.length).toBeGreaterThan(0)
  })
})
