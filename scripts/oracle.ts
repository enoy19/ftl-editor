#!/usr/bin/env bun
// Checks a file against the Java editor: Vhati's FTL Profile Editor
// (https://github.com/Vhati/ftl-profile-editor) reads it, writes it back to
// memory and compares the bytes. A file written by `ftled` must pass.
//
//   bun scripts/oracle.ts <file.sav> [--resources <dir>]
//   bun scripts/oracle.ts --profile [<ae_prof.sav>] [--resources <dir>]
//
// <file.sav>    a saved game (continue.sav); with --profile a profile
//               (ae_prof.sav or prof.sav); without one, --profile prints the
//               ship table only
// --profile     run ProfileRoundTrip instead of SaveRoundTrip
// --resources   the folder that holds ftl.dat (on macOS
//               <FTL.app>/Contents/Resources); default: the ftl.dat the app
//               remembered, else the first existing install path the app
//               searches (Steam, GOG, Humble; Wine on Linux)
// --help        print this text
//
// Needs git, Java 21 or later and Maven on the PATH, and FTL installed.
//
// Steps:
// 1. Submodule reference/ftl-profile-editor (the public upstream, pinned by
//    the gitlink): `git submodule update --init` when it is not checked out,
//    or when it stands on another commit with a clean tree. Another commit
//    with local changes: refuse, both commits named.
// 2. The patches reference/patches/*.patch in name order, each idempotent:
//    `git apply --check` passes -> apply it; only `git apply --check
//    --reverse` passes -> already applied, skip; neither -> refuse with both
//    git messages. 0001 builds the editor on Java 21 and adds SaveRoundTrip,
//    0002 adds ProfileRoundTrip.
// 3. Toolchain: `java -version` and the Java of `mvn -v` must be 21 or later.
// 4. Build: `mvn -q -B package -DskipTests` in the submodule
//    (target/FTLProfileEditor.jar, with all dependencies).
// 5. Probe: `java -cp <jar> net.blerf.ftl.tools.SaveRoundTrip <resources>
//    <file>` (or ProfileRoundTrip). It prints what it read and
//    `IDENTICAL: <n> bytes` or `DIFFER ...`.
//
// Output: one line per step (`oracle: <step>`), then the probe's own lines.
// Exit code: the probe's verdict (0 IDENTICAL, 1 DIFFER, 2 its usage or read
// error); 2 for a usage error of this script; 3 when a step before the probe
// refused (each refusal names the cause and the fix).

import { spawnSync } from 'node:child_process'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { parseArgs } from 'node:util'

const ROOT = resolve(import.meta.dir, '..')
const SUBMODULE = 'reference/ftl-profile-editor'
const PATCHES = 'reference/patches'
const MIN_JAVA = 21

class Refusal extends Error {}

function log(step: string): void {
  console.log(`oracle: ${step}`)
}

/** The major version in `java -version` output ("1.8.0" -> 8, "21.0.10" -> 21). */
export function parseJavaMajor(text: string): number | undefined {
  const m = text.match(/version "(\d+)(?:\.(\d+))?[^"]*"/)
  if (!m) return undefined
  const first = Number(m[1])
  return first === 1 && m[2] !== undefined ? Number(m[2]) : first
}

/** The major version of the Java that Maven runs on, from `mvn -v`. */
export function parseMavenJavaMajor(text: string): number | undefined {
  const m = text.match(/^Java version: (\d+)(?:\.(\d+))?/m)
  if (!m) return undefined
  const first = Number(m[1])
  return first === 1 && m[2] !== undefined ? Number(m[2]) : first
}

export type PatchState = 'unapplied' | 'applied' | 'conflict'

/** The state of one patch from `git apply --check` forward and `--reverse`. */
export function patchState(forwardApplies: boolean, reverseApplies: boolean): PatchState {
  if (forwardApplies) return 'unapplied'
  if (reverseApplies) return 'applied'
  return 'conflict'
}

export interface Env {
  os: string // process.platform
  home: string
  programFilesX86?: string
  programFiles?: string
  xdgDataHome?: string
  winePrefix?: string
}

/**
 * The folders that may hold ftl.dat, in the order of `candidates_for` in
 * crates/ftl-app/src/assets.rs (itself the list of the Java editor).
 */
export function datsDirCandidates(env: Env): string[] {
  const STEAM = 'Steam/steamapps/common/FTL Faster Than Light'
  const STEAM_OLD_CASE = 'Steam/SteamApps/common/FTL Faster Than Light'
  const GOG = 'GOG.com/Faster Than Light'
  const HUMBLE = 'FTL'
  const dirs: string[] = []
  if (env.os === 'darwin') {
    for (const steam of [STEAM, STEAM_OLD_CASE]) {
      dirs.push(join(env.home, 'Library/Application Support', steam, 'FTL.app/Contents/Resources'))
    }
    dirs.push('/Applications/FTL.app/Contents/Resources')
  } else if (env.os === 'win32') {
    for (const base of [env.programFilesX86 ?? 'C:\\Program Files (x86)', env.programFiles ?? 'C:\\Program Files']) {
      for (const sub of ['', 'resources']) {
        for (const store of [STEAM, GOG, HUMBLE]) dirs.push(join(base, sub, store))
      }
    }
  } else {
    const xdg = env.xdgDataHome ?? join(env.home, '.local/share')
    for (const steamRoot of [join(xdg, 'Steam'), join(env.home, '.steam/steam')]) {
      for (const sub of ['data', 'data/resources']) {
        for (const apps of ['steamapps', 'SteamApps']) {
          dirs.push(join(steamRoot, apps, 'common/FTL Faster Than Light', sub))
        }
      }
    }
    const wine = env.winePrefix ?? join(env.home, '.wine')
    for (const sub of ['', 'resources']) {
      for (const programFiles of ['Program Files (x86)', 'Program Files']) {
        for (const store of [GOG, HUMBLE]) dirs.push(join(wine, 'drive_c', programFiles, store, sub))
      }
    }
  }
  return dirs
}

/** The folder of the ftl.dat in the app's config.json, if it names one. */
export function rememberedDatsDir(configJson: string): string | undefined {
  try {
    const path = (JSON.parse(configJson) as { ftl_dat?: unknown }).ftl_dat
    return typeof path === 'string' && path !== '' ? dirname(path) : undefined
  } catch {
    return undefined
  }
}

/** The app's config.json (`directories::ProjectDirs::from("", "", "ftl-editor")`). */
function appConfigFile(env: Env): string {
  if (env.os === 'darwin') return join(env.home, 'Library/Application Support/ftl-editor/config.json')
  if (env.os === 'win32') return join(process.env.APPDATA ?? join(env.home, 'AppData/Roaming'), 'ftl-editor/config/config.json')
  return join(process.env.XDG_CONFIG_HOME ?? join(env.home, '.config'), 'ftl-editor/config.json')
}

function currentEnv(): Env {
  return {
    os: process.platform,
    home: homedir(),
    programFilesX86: process.env['ProgramFiles(x86)'],
    programFiles: process.env.ProgramFiles,
    xdgDataHome: process.env.XDG_DATA_HOME,
    winePrefix: process.env.WINEPREFIX,
  }
}

function locateResources(flag: string | undefined): string {
  if (flag !== undefined) {
    const dir = resolve(flag)
    if (!existsSync(join(dir, 'ftl.dat'))) throw new Refusal(`--resources ${dir} holds no ftl.dat`)
    return dir
  }
  const env = currentEnv()
  const config = appConfigFile(env)
  if (existsSync(config)) {
    const remembered = rememberedDatsDir(readFileSync(config, 'utf8'))
    if (remembered && existsSync(join(remembered, 'ftl.dat'))) return remembered
  }
  const tried = datsDirCandidates(env)
  const found = tried.find((d) => existsSync(join(d, 'ftl.dat')))
  if (found) return found
  throw new Refusal(`no ftl.dat found; pass --resources <dir>. Tried:\n  ${tried.join('\n  ')}`)
}

function run(cmd: string, args: string[], cwd: string): { status: number | null; out: string; err: string; missing: boolean } {
  const r = spawnSync(cmd, args, { cwd, maxBuffer: 1 << 28 })
  const missing = (r.error as NodeJS.ErrnoException | undefined)?.code === 'ENOENT'
  return { status: r.status, out: r.stdout?.toString() ?? '', err: r.stderr?.toString() ?? '', missing }
}

function must(cmd: string, args: string[], cwd: string): string {
  const r = run(cmd, args, cwd)
  if (r.missing) throw new Refusal(`${cmd} is not on the PATH`)
  if (r.status !== 0) throw new Refusal(`${cmd} ${args.join(' ')} exited ${r.status}: ${(r.err || r.out).trim()}`)
  return r.out.trim()
}

/** Step 1. */
function submodule(): string {
  const sub = join(ROOT, SUBMODULE)
  const pinned = must('git', ['ls-tree', 'HEAD', SUBMODULE], ROOT).split(/\s+/)[2]
  if (!pinned) throw new Refusal(`HEAD has no gitlink ${SUBMODULE}`)
  const checkedOut = existsSync(join(sub, '.git'))
  const head = checkedOut ? run('git', ['rev-parse', 'HEAD'], sub).out.trim() : ''
  if (head === pinned) {
    log(`submodule at ${pinned.slice(0, 12)}`)
    return sub
  }
  if (checkedOut && must('git', ['status', '--porcelain'], sub) !== '') {
    throw new Refusal(
      `${SUBMODULE} stands on ${head.slice(0, 12)} with local changes, the gitlink pins ${pinned.slice(0, 12)}; ` +
        `commit or remove those changes, then run: git submodule update ${SUBMODULE}`,
    )
  }
  log(`git submodule update --init ${SUBMODULE} (pinned ${pinned.slice(0, 12)})`)
  must('git', ['submodule', 'update', '--init', '--', SUBMODULE], ROOT)
  const now = must('git', ['rev-parse', 'HEAD'], sub)
  if (now !== pinned) throw new Refusal(`${SUBMODULE} is at ${now.slice(0, 12)} after the update, the gitlink pins ${pinned.slice(0, 12)}`)
  return sub
}

/** Step 2. */
function patches(sub: string): void {
  const dir = join(ROOT, PATCHES)
  const files = readdirSync(dir).filter((f) => f.endsWith('.patch')).sort()
  if (files.length === 0) throw new Refusal(`${PATCHES} holds no *.patch`)
  for (const f of files) {
    const p = join(dir, f)
    const forward = run('git', ['apply', '--check', p], sub)
    const reverse = run('git', ['apply', '--check', '--reverse', p], sub)
    const state = patchState(forward.status === 0, reverse.status === 0)
    if (state === 'applied') {
      log(`patch ${f}: already applied`)
    } else if (state === 'unapplied') {
      must('git', ['apply', p], sub)
      log(`patch ${f}: applied`)
    } else {
      throw new Refusal(
        `patch ${f} neither applies nor is applied in ${SUBMODULE}; is the submodule at the pinned commit?\n` +
          `  git apply --check: ${forward.err.trim()}\n  git apply --check --reverse: ${reverse.err.trim()}`,
      )
    }
  }
}

/** Step 3. */
function toolchain(sub: string): void {
  const java = run('java', ['-version'], sub)
  if (java.missing) throw new Refusal(`java is not on the PATH; install a JDK ${MIN_JAVA} or later`)
  const javaMajor = parseJavaMajor(java.err + java.out)
  if (javaMajor === undefined) throw new Refusal(`java -version answered no version: ${(java.err + java.out).trim()}`)
  if (javaMajor < MIN_JAVA) throw new Refusal(`java on the PATH is Java ${javaMajor}; the patched editor needs Java ${MIN_JAVA} or later`)
  const mvn = run('mvn', ['-v'], sub)
  if (mvn.missing) throw new Refusal('mvn is not on the PATH; install Apache Maven')
  const mvnMajor = parseMavenJavaMajor(mvn.out + mvn.err)
  if (mvnMajor === undefined) throw new Refusal(`mvn -v named no Java version: ${(mvn.out + mvn.err).trim()}`)
  if (mvnMajor < MIN_JAVA) throw new Refusal(`mvn runs on Java ${mvnMajor} (JAVA_HOME); set JAVA_HOME to a JDK ${MIN_JAVA} or later`)
  log(`java ${javaMajor}, mvn on java ${mvnMajor}`)
}

/** Step 4. */
function build(sub: string): string {
  log('mvn -q -B package -DskipTests')
  const r = spawnSync('mvn', ['-q', '-B', 'package', '-DskipTests'], { cwd: sub, stdio: 'inherit' })
  if (r.status !== 0) throw new Refusal(`mvn package exited ${r.status ?? r.signal} in ${SUBMODULE}`)
  const jar = join(sub, 'target/FTLProfileEditor.jar')
  if (!existsSync(jar)) throw new Refusal(`mvn package wrote no ${SUBMODULE}/target/FTLProfileEditor.jar`)
  return jar
}

const HELP = readFileSync(import.meta.path, 'utf8')
  .split('\n')
  .slice(1)
  .filter((l) => l.startsWith('//'))
  .map((l) => l.replace(/^\/\/ ?/, ''))
  .join('\n')

export function main(argv: string[]): number {
  let values: { profile?: boolean; resources?: string; help?: boolean }
  let positionals: string[]
  try {
    ;({ values, positionals } = parseArgs({
      args: argv,
      options: { profile: { type: 'boolean' }, resources: { type: 'string' }, help: { type: 'boolean' } },
      allowPositionals: true,
    }))
  } catch (e) {
    console.error(`oracle: ${(e as Error).message}\n\n${HELP}`)
    return 2
  }
  if (values.help) {
    console.log(HELP)
    return 0
  }
  const needsFile = !values.profile
  if (positionals.length > 1 || (needsFile && positionals.length !== 1)) {
    console.error(`oracle: give ${needsFile ? 'exactly one <file.sav>' : 'at most one <ae_prof.sav>'}\n\n${HELP}`)
    return 2
  }
  const file = positionals[0] === undefined ? undefined : resolve(positionals[0])
  if (file !== undefined && !existsSync(file)) {
    console.error(`oracle: ${file} does not exist`)
    return 2
  }
  try {
    const resources = locateResources(values.resources)
    log(`resources ${resources}`)
    const sub = submodule()
    patches(sub)
    toolchain(sub)
    const jar = build(sub)
    const tool = values.profile ? 'ProfileRoundTrip' : 'SaveRoundTrip'
    log(`${tool} ${file ?? '(ship table only)'}`)
    const args = ['-cp', jar, `net.blerf.ftl.tools.${tool}`, resources, ...(file === undefined ? [] : [file])]
    // The editor writes ftl-editor-log.txt into its working directory; the
    // submodule's patched .gitignore ignores it.
    const probe = spawnSync('java', args, { cwd: sub, stdio: 'inherit' })
    return probe.status ?? 1
  } catch (error) {
    if (error instanceof Refusal) {
      console.error(`oracle: ${error.message}`)
      return 3
    }
    throw error
  }
}

if (import.meta.main) process.exit(main(process.argv.slice(2)))
