#!/usr/bin/env bun
// Builds the macOS app and packs it into a DMG installer.
//
//   bun scripts/build-dmg.ts [--no-build] [--generated <tar>] [--verify] [--no-receipt]
//
// --no-build    pack the app of the last build in the build folder (step 2) as it
//               is; default: export and `flutter build macos --release` first
// --generated   unpack the generated files from <tar> (written by
//               `scripts/codegen.ts --pack`) into the export instead of running
//               scripts/codegen.ts there; CI generates once for every job
// --verify      after the build: attach the DMG read-only, check the icon of the
//               app inside against app_icon_256.png, start the app with
//               `open -n`, check its process, end only that process, detach
// --no-receipt  write no receipt
// --help        print this text
//
// Steps:
// 1. Refuse another host than macOS on arm64 (the DMG name says arm64).
// 2. `git archive HEAD` into the build folder <tmpdir>/ftl-editor-release-src
//    (emptied first; uncommitted changes are not built, a warning names
//    them), then the generated files (`bun scripts/codegen.ts --root <export>`
//    or the tar of --generated; they are not in git), then `flutter build
//    macos --release` in its app/ (it also builds the Rust bridge) with
//    PUB_CACHE=<tmpdir>/ftl-editor-release-pub-cache.
//    Why: the Swift runner, the plugin objects and the Dart snapshot keep the
//    absolute paths of their sources; a build in the checkout under the home
//    folder would publish them. app/hook/build.dart remaps the Rust paths.
// 3. Sign ad-hoc: `codesign --force --deep --sign -`, then `codesign --verify
//    --deep --strict`. The first builds are not notarized: Gatekeeper asks on
//    the first start (right click, Open). Then scripts/scan-binaries.ts scans
//    every file of the app for paths and names of this machine; a hit stops
//    the build before the DMG exists.
// 4. A staging folder with the app and a symlink to /Applications, then
//    `hdiutil create -format UDZO` to dist/FTL-Save-Editor-<version>-macos-arm64.dmg.
//    <version> is the `version:` of app/pubspec.yaml without the build number.
//    dist/ is gitignored.
// 5. The receipt dist/<date>-dmg-macos-arm64.json (gitignored): source commit and
//    whether the tree was clean, the command, the tool versions, size and
//    sha256 of the DMG, and the verify result when --verify ran.
//
// Output: one line per step (`build-dmg: <step>`), then the DMG path. Exit code
// 0 after a build, 1 when a step failed (its command and exit code named), 2 for
// a usage error or a refused host.

import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { arch, platform, tmpdir } from 'node:os'
import { join, relative, resolve } from 'node:path'
import { parseArgs } from 'node:util'
import { scanPaths } from './scan-binaries.ts'

const ROOT = resolve(import.meta.dir, '..')
const APP_DIR = join(ROOT, 'app')
const APP_NAME = 'FTL Save Editor'
// The build folder outside the home folder (step 2 of the help).
const BUILD_ROOT = join(realpathSync(tmpdir()), 'ftl-editor-release-src')
const PUB_CACHE = join(realpathSync(tmpdir()), 'ftl-editor-release-pub-cache')
const RELEASE_APP = join(BUILD_ROOT, 'app/build/macos/Build/Products/Release', `${APP_NAME}.app`)
// actool puts the sizes up to 256 px into AppIcon.icns, the larger ones into
// Assets.car only; 256 px is the largest image both hold.
const ICON_256 = join(APP_DIR, 'macos/Runner/Assets.xcassets/AppIcon.appiconset/app_icon_256.png')

class StepError extends Error {}

function log(step: string): void {
  console.log(`build-dmg: ${step}`)
}

/** Runs a command; throws a StepError that names it and its exit code. */
function run(cmd: string, args: string[], opts: { cwd?: string; quiet?: boolean; env?: NodeJS.ProcessEnv; input?: Buffer } = {}): string {
  const res = spawnSync(cmd, args, {
    cwd: opts.cwd ?? ROOT,
    env: opts.env ?? process.env,
    input: opts.input,
    encoding: 'utf8',
    stdio: opts.quiet === false ? 'inherit' : 'pipe',
    maxBuffer: 64 * 1024 * 1024,
  })
  if (res.status !== 0) {
    const out = `${res.stdout ?? ''}${res.stderr ?? ''}`.trim()
    throw new StepError(`\`${[cmd, ...args].join(' ')}\` exited with ${res.status ?? res.signal}\n${out}`)
  }
  return (res.stdout ?? '').trim()
}

function sha256(path: string): string {
  return createHash('sha256').update(readFileSync(path)).digest('hex')
}

/** The version of app/pubspec.yaml without the build number. */
export function appVersion(pubspec: string): string {
  const match = pubspec.match(/^version:\s*([0-9]+\.[0-9]+\.[0-9]+)/m)
  if (!match) throw new StepError('app/pubspec.yaml has no `version: x.y.z`')
  return match[1]
}

export function dmgName(version: string): string {
  return `FTL-Save-Editor-${version}-macos-arm64.dmg`
}

function pixels(png: string, scratch: string, name: string): Buffer {
  const bmp = join(scratch, `${name}.bmp`)
  // sips exits 0 and writes nothing for a missing or broken input.
  if (!existsSync(png)) throw new StepError(`no image at ${png}`)
  run('sips', ['-s', 'format', 'bmp', png, '--out', bmp])
  if (!existsSync(bmp)) throw new StepError(`sips wrote no bitmap of ${png}`)
  return readFileSync(bmp)
}

function sleep(ms: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms)
}

/** The PIDs of processes whose command line starts inside `folder`. */
function pidsIn(folder: string): number[] {
  const res = spawnSync('pgrep', ['-f', `^${folder}/`], { encoding: 'utf8' })
  return res.stdout
    .split('\n')
    .filter((line) => line.trim() !== '')
    .map(Number)
}

type Verify = { icon: string; launched: boolean; pid?: number; endedBy?: string; detached: boolean }

/** Mounts the DMG read-only, checks the icon and a start of the app, detaches. */
function verify(dmg: string): Verify {
  // realpath: tmpdir() is /var/..., the process list shows /private/var/...
  const scratch = realpathSync(mkdtempSync(join(tmpdir(), 'ftl-editor-dmg-verify-')))
  const mount = join(scratch, 'mount')
  mkdirSync(mount)
  run('hdiutil', ['attach', '-nobrowse', '-readonly', '-mountpoint', mount, dmg])
  const result: Verify = { icon: 'unchecked', launched: false, detached: false }
  try {
    const app = join(mount, `${APP_NAME}.app`)
    if (!existsSync(app)) throw new StepError(`no ${APP_NAME}.app in the DMG`)
    if (!existsSync(join(mount, 'Applications'))) throw new StepError('no Applications link in the DMG')
    // actool compiles the appiconset into AppIcon.icns; its 256 px image must
    // have the pixels of the committed app_icon_256.png.
    const iconset = join(scratch, 'AppIcon.iconset')
    run('iconutil', ['-c', 'iconset', join(app, 'Contents/Resources/AppIcon.icns'), '-o', iconset])
    const same = pixels(join(iconset, 'icon_128x128@2x.png'), scratch, 'bundled').equals(pixels(ICON_256, scratch, 'source'))
    result.icon = same ? 'matches app_icon_256.png' : 'differs from app_icon_256.png'
    if (!same) throw new StepError('the icon in the DMG differs from app_icon_256.png')

    run('open', ['-n', app])
    let pid: number | undefined
    for (let i = 0; i < 40 && pid === undefined; i++) {
      sleep(250)
      pid = pidsIn(app)[0]
    }
    if (pid === undefined) throw new StepError('the app from the DMG did not start within 10 s')
    sleep(3000)
    result.launched = pidsIn(app).includes(pid)
    result.pid = pid
    if (!result.launched) throw new StepError(`the app from the DMG (PID ${pid}) ended within 3 s`)
    // Only this PID is ended: it runs from the private mount point, so it is
    // never an app the user started. SIGKILL follows when SIGTERM has not
    // ended it within 5 s.
    result.endedBy = 'SIGTERM'
    process.kill(pid, 'SIGTERM')
    for (let i = 0; i < 20 && pidsIn(app).includes(pid); i++) sleep(250)
    if (pidsIn(app).includes(pid)) {
      result.endedBy = 'SIGKILL'
      process.kill(pid, 'SIGKILL')
      for (let i = 0; i < 20 && pidsIn(app).includes(pid); i++) sleep(250)
    }
    if (pidsIn(app).includes(pid)) throw new StepError(`PID ${pid} from the DMG did not end`)
    return result
  } finally {
    for (let i = 0; i < 5 && !result.detached; i++) {
      if (i > 0) sleep(1000)
      result.detached = spawnSync('hdiutil', ['detach', mount], { encoding: 'utf8' }).status === 0
    }
    // A failed detach keeps the scratch folder: removing it would recurse into
    // the read-only mount (EROFS).
    if (result.detached) rmSync(scratch, { recursive: true, force: true })
    else console.error(`build-dmg: detach failed, still mounted at ${mount}`)
  }
}

function toolVersions(): Record<string, string> {
  const tool = (cmd: string, args: string[]) => {
    const res = spawnSync(cmd, args, { encoding: 'utf8' })
    return res.status === 0 ? res.stdout.trim().split('\n')[0] : 'unavailable'
  }
  return {
    macos: tool('sw_vers', ['-productVersion']),
    flutter: tool('flutter', ['--version']),
    rustc: tool('rustc', ['--version']),
    xcode: tool('xcodebuild', ['-version']),
    bun: Bun.version,
  }
}

function main(): number {
  const { values } = parseArgs({
    options: {
      'no-build': { type: 'boolean', default: false },
      generated: { type: 'string' },
      verify: { type: 'boolean', default: false },
      'no-receipt': { type: 'boolean', default: false },
      help: { type: 'boolean', default: false },
    },
  })
  if (values.help) {
    const head = readFileSync(import.meta.path, 'utf8').split('\n\n')[0]
    console.log(head.replace(/^#!.*\n/, '').replace(/^\/\/ ?/gm, ''))
    return 0
  }
  if (platform() !== 'darwin' || arch() !== 'arm64') {
    console.error(`build-dmg: needs macOS on arm64, this is ${platform()} ${arch()}`)
    return 2
  }
  try {
    const version = appVersion(readFileSync(join(APP_DIR, 'pubspec.yaml'), 'utf8'))
    if (!values['no-build']) {
      const dirty = run('git', ['status', '--porcelain'])
      if (dirty !== '') console.error(`build-dmg: warning: uncommitted changes are not built:\n${dirty}`)
      log(`export HEAD to ${BUILD_ROOT}`)
      rmSync(BUILD_ROOT, { recursive: true, force: true })
      mkdirSync(BUILD_ROOT, { recursive: true })
      const tar = spawnSync('git', ['archive', '--format=tar', 'HEAD'], { cwd: ROOT, maxBuffer: 1 << 30 })
      if (tar.status !== 0) throw new StepError(`git archive exited ${tar.status}: ${tar.stderr.toString().trim()}`)
      run('tar', ['-x', '-f', '-', '-C', BUILD_ROOT], { input: tar.stdout })
      if (values.generated) {
        log(`unpack the generated files of ${values.generated}`)
        run('tar', ['-x', '-f', resolve(values.generated), '-C', BUILD_ROOT])
      } else {
        log('bun scripts/codegen.ts')
        // The export's own script, so the generated files follow the exported source.
        run('bun', [join(BUILD_ROOT, 'scripts/codegen.ts'), '--root', BUILD_ROOT], { quiet: false })
      }
      log('flutter build macos --release')
      run('flutter', ['build', 'macos', '--release'], {
        cwd: join(BUILD_ROOT, 'app'),
        quiet: false,
        env: { ...process.env, PUB_CACHE },
      })
    }
    if (!existsSync(RELEASE_APP)) throw new StepError(`no app at ${RELEASE_APP}`)

    log('codesign ad-hoc')
    run('codesign', ['--force', '--deep', '--sign', '-', RELEASE_APP])
    run('codesign', ['--verify', '--deep', '--strict', RELEASE_APP])

    log('scan the app for paths and names of this machine')
    const hits = scanPaths([RELEASE_APP])
    if (hits > 0) throw new StepError(`scan-binaries: ${hits} hit(s) in ${RELEASE_APP}; no DMG`)

    log('hdiutil create')
    const dist = join(ROOT, 'dist')
    mkdirSync(dist, { recursive: true })
    const dmg = join(dist, dmgName(version))
    const staging = mkdtempSync(join(tmpdir(), 'ftl-editor-dmg-'))
    try {
      // ditto keeps the symlinks and extended attributes of the bundle.
      run('ditto', [RELEASE_APP, join(staging, `${APP_NAME}.app`)])
      symlinkSync('/Applications', join(staging, 'Applications'))
      run('hdiutil', ['create', '-volname', `${APP_NAME} ${version}`, '-srcfolder', staging, '-ov', '-format', 'UDZO', dmg])
    } finally {
      rmSync(staging, { recursive: true, force: true })
    }

    let verified: Verify | undefined
    if (values.verify) {
      log('verify')
      verified = verify(dmg)
      log(`verify: icon ${verified.icon}, started as PID ${verified.pid}, ended by ${verified.endedBy}, detached ${verified.detached}`)
    }

    if (!values['no-receipt']) {
      const commit = run('git', ['rev-parse', 'HEAD'])
      const clean = run('git', ['status', '--porcelain']) === ''
      const date = new Date().toISOString().slice(0, 10)
      const receipts = join(ROOT, 'dist')
      mkdirSync(receipts, { recursive: true })
      const receipt = join(receipts, `${date}-dmg-macos-arm64.json`)
      const body = {
        date: new Date().toISOString(),
        command: `bun scripts/build-dmg.ts ${process.argv.slice(2).join(' ')}`.trim(),
        source: { commit, clean },
        version,
        signing: 'ad-hoc (codesign --force --deep --sign -), not notarized',
        tools: toolVersions(),
        outputs: [{ path: relative(ROOT, dmg), bytes: statSync(dmg).size, sha256: sha256(dmg) }],
        verify: verified ?? null,
      }
      writeFileSync(receipt, `${JSON.stringify(body, null, 2)}\n`)
      log(`receipt ${relative(ROOT, receipt)}`)
    }
    console.log(`build-dmg: ${dmg}`)
    return 0
  } catch (error) {
    if (error instanceof StepError) {
      console.error(`build-dmg: ${error.message}`)
      return 1
    }
    throw error
  }
}

if (import.meta.main) process.exit(main())
