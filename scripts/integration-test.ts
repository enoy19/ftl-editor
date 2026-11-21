#!/usr/bin/env bun
// Runs the Flutter integration tests of app/ on macOS, one run per machine at a time.
//
//   bun scripts/integration-test.ts [integration_test/<file>.dart ...] [extra flutter test args]
//
// Without a file it runs every file under app/integration_test (the gate).
// With one or more files (paths relative to app/) it runs only those: the
// narrow run while a change is under work.
//
// Why: an integration test drives a real app window. Two runs at the same time
// (two worktrees, two agents) take the keyboard focus from each other, and
// enterText into a popover field then lands nowhere: the tests fail at random
// with values that never arrived. The lock serializes the runs of every
// checkout on this machine.
//
// The lock is a directory under the OS temp folder, created atomically with
// mkdir. It holds the PID of the owner; a lock whose PID no longer runs is
// stale and is taken over. Waiting prints who holds the lock every 30 s.
//
// Before the lock it runs scripts/codegen.ts: the bindings and localizations
// are not in git (AGENTS.md, "Generated files").
// Exit code: the exit code of scripts/codegen.ts when it fails, else the exit
// code of `flutter test`.

import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

const lock = join(tmpdir(), 'ftl-editor-integration-test.lock')
const pidFile = join(lock, 'pid')
const appDir = resolve(import.meta.dir, '..', 'app')

const codegen = Bun.spawnSync(['bun', resolve(import.meta.dir, 'codegen.ts')], { stdout: 'inherit', stderr: 'inherit' })
if (codegen.exitCode !== 0) process.exit(codegen.exitCode ?? 1)

function alive(pid: number): boolean {
  try {
    process.kill(pid, 0)
    return true
  } catch {
    return false
  }
}

function tryTake(): boolean {
  try {
    mkdirSync(lock)
    writeFileSync(pidFile, `${process.pid}\n${appDir}\n`)
    return true
  } catch {
    let owner = Number.NaN
    try {
      owner = Number(readFileSync(pidFile, 'utf8').split('\n')[0])
    } catch {
      // Lock dir without pid file: its owner is between mkdir and write.
      return false
    }
    if (Number.isFinite(owner) && !alive(owner)) {
      console.error(`integration-test: taking over a stale lock of PID ${owner}`)
      rmSync(lock, { recursive: true, force: true })
      return tryTake()
    }
    return false
  }
}

let waited = 0
while (!tryTake()) {
  if (waited % 30 === 0) {
    let holder = 'unknown'
    try {
      holder = readFileSync(pidFile, 'utf8').trim().replace('\n', ' in ')
    } catch {}
    console.error(`integration-test: waiting for the lock (${lock}), held by PID ${holder}`)
  }
  await Bun.sleep(1000)
  waited++
}

const release = () => rmSync(lock, { recursive: true, force: true })
process.on('SIGINT', () => {
  release()
  process.exit(130)
})
process.on('SIGTERM', () => {
  release()
  process.exit(143)
})

const args = process.argv.slice(2)
const files = args.filter((a) => a.endsWith('.dart'))
const rest = args.filter((a) => !a.endsWith('.dart'))
const targets = files.length > 0 ? files : ['integration_test']
console.error(`integration-test: running ${targets.join(' ')}`)
const run = Bun.spawn(['flutter', 'test', ...targets, '-d', 'macos', ...rest], {
  cwd: appDir,
  stdout: 'inherit',
  stderr: 'inherit',
})
const code = await run.exited
release()
process.exit(code)
