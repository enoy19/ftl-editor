#!/usr/bin/env bun
// Generates the files that are not in git: the flutter_rust_bridge bindings
// and the Flutter localizations. Every build path runs it before it needs them.
//
//   bun scripts/codegen.ts [--root <dir>] [--install] [--pack <tar>]
//   bun scripts/codegen.ts --print-version
//
// --root           the repository to generate in (default: the checkout of
//                  this script); build-dmg passes its export
// --install        first `cargo install flutter_rust_bridge_codegen --version
//                  <pin> --locked` when the installed codegen is missing or
//                  has another version
// --pack <tar>     afterwards write the generated files into <tar> (paths
//                  relative to the root); CI hands this tar to its other jobs
// --print-version  print the pinned flutter_rust_bridge version and exit
// --help           print this text
//
// The pin is the version of the crate flutter_rust_bridge in Cargo.lock. The
// package flutter_rust_bridge in app/pubspec.lock must carry the same version,
// and `flutter_rust_bridge_codegen --version` must answer it; otherwise the
// script refuses before it writes anything.
//
// Steps:
// 1. `flutter_rust_bridge_codegen generate` in <root>/app (app/flutter_rust_bridge.yaml):
//    app/rust/src/frb_generated.rs and app/lib/src/rust/**. Needs Rust and Flutter.
//    rustfmt skips frb_generated.rs (`#[rustfmt::skip]` in app/rust/src/lib.rs),
//    so no `cargo fmt` follows.
// 2. `flutter gen-l10n` in <root>/app (app/l10n.yaml): app/lib/l10n/app_localizations*.dart.
//    `flutter pub get` also runs it (`generate: true` in app/pubspec.yaml).
//
// Output: one line per step (`codegen: <step>`). Exit code 0 after both steps,
// 1 when a step failed or a version differs (both named), 2 for a usage error.

import { spawnSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { parseArgs } from 'node:util'

const ROOT = resolve(import.meta.dir, '..')

/** The generated paths, relative to the repository root; .gitignore lists the same. */
export const GENERATED = ['app/rust/src/frb_generated.rs', 'app/lib/src/rust', 'app/lib/l10n/app_localizations*.dart']

class StepError extends Error {}

function log(step: string): void {
  console.log(`codegen: ${step}`)
}

/** The version of `name` in a Cargo.lock. */
export function cargoLockVersion(lock: string, name: string): string | undefined {
  const re = new RegExp(`\\[\\[package\\]\\]\\nname = "${name}"\\nversion = "([^"]+)"`)
  return lock.match(re)?.[1]
}

/** The version of the hosted package `name` in a pubspec.lock. */
export function pubspecLockVersion(lock: string, name: string): string | undefined {
  const re = new RegExp(`\\n  ${name}:\\n(?:    .*\\n|      .*\\n)*?    version: "([^"]+)"`)
  return lock.match(re)?.[1]
}

/** The pinned flutter_rust_bridge version of `root`; refuses when the two lockfiles disagree. */
export function pinnedVersion(root: string): string {
  const cargo = cargoLockVersion(readFileSync(join(root, 'Cargo.lock'), 'utf8'), 'flutter_rust_bridge')
  if (!cargo) throw new StepError('Cargo.lock has no package flutter_rust_bridge')
  const pub = pubspecLockVersion(readFileSync(join(root, 'app/pubspec.lock'), 'utf8'), 'flutter_rust_bridge')
  if (!pub) throw new StepError('app/pubspec.lock has no package flutter_rust_bridge')
  if (cargo !== pub) {
    throw new StepError(`flutter_rust_bridge differs: Cargo.lock ${cargo}, app/pubspec.lock ${pub}`)
  }
  return cargo
}

/** The version `flutter_rust_bridge_codegen --version` answers, or undefined when it does not run. */
function installedCodegen(): string | undefined {
  const res = spawnSync('flutter_rust_bridge_codegen', ['--version'], { encoding: 'utf8' })
  if (res.status !== 0) return undefined
  return res.stdout.trim().split(/\s+/).pop()
}

function run(cmd: string, args: string[], cwd: string): void {
  log([cmd, ...args].join(' '))
  const res = spawnSync(cmd, args, { cwd, stdio: 'inherit', shell: process.platform === 'win32' })
  if (res.error) throw new StepError(`\`${[cmd, ...args].join(' ')}\` did not start: ${res.error.message}`)
  if (res.status !== 0) throw new StepError(`\`${[cmd, ...args].join(' ')}\` exited with ${res.status ?? res.signal}`)
}

function main(): number {
  let values: { root?: string; install: boolean; pack?: string; 'print-version': boolean; help: boolean }
  try {
    values = parseArgs({
      options: {
        root: { type: 'string' },
        install: { type: 'boolean', default: false },
        pack: { type: 'string' },
        'print-version': { type: 'boolean', default: false },
        help: { type: 'boolean', default: false },
      },
    }).values as typeof values
  } catch (error) {
    console.error(`codegen: ${(error as Error).message}`)
    return 2
  }
  if (values.help) {
    const head = readFileSync(import.meta.path, 'utf8').split('\n\n')[0]
    console.log(head.replace(/^#!.*\n/, '').replace(/^\/\/ ?/gm, ''))
    return 0
  }
  const root = resolve(values.root ?? ROOT)
  if (!existsSync(join(root, 'app/flutter_rust_bridge.yaml'))) {
    console.error(`codegen: ${root} is no ftl-editor checkout (no app/flutter_rust_bridge.yaml)`)
    return 2
  }
  try {
    const pin = pinnedVersion(root)
    if (values['print-version']) {
      console.log(pin)
      return 0
    }
    let installed = installedCodegen()
    if (installed !== pin && values.install) {
      run('cargo', ['install', 'flutter_rust_bridge_codegen', '--version', pin, '--locked'], root)
      installed = installedCodegen()
    }
    if (installed !== pin) {
      throw new StepError(
        `flutter_rust_bridge_codegen ${installed ?? 'is not installed'}${installed ? ' is installed' : ''}, the pin is ${pin}; ` +
          `run \`cargo install flutter_rust_bridge_codegen --version ${pin} --locked\` (or pass --install)`,
      )
    }
    const app = join(root, 'app')
    run('flutter_rust_bridge_codegen', ['generate'], app)
    run('flutter', ['gen-l10n'], app)
    if (values.pack) {
      const tar = resolve(values.pack)
      // bash expands the glob of the l10n files; tar keeps the paths relative to the root.
      const res = spawnSync('bash', ['-c', `tar -cf "$0" ${GENERATED.join(' ')}`, tar], { cwd: root, stdio: 'inherit' })
      if (res.status !== 0) throw new StepError(`tar of the generated files exited with ${res.status ?? res.signal}`)
      log(`packed ${tar}`)
    }
    log(`done in ${root}`)
    return 0
  } catch (error) {
    if (error instanceof StepError) {
      console.error(`codegen: ${error.message}`)
      return 1
    }
    throw error
  }
}

if (import.meta.main) process.exit(main())
