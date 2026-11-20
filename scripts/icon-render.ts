#!/usr/bin/env bun
// Renders the app icon from brand/logo.svg to the icons of the three desktop shells.
//
//   bun scripts/icon-render.ts [--source <svg>] [--check]
//
// --source  the SVG (default: brand/logo.svg). It is the only hand-edited source;
//           brand/logo-*.svg are the variants, and a swap is one copy.
// --check   renders into a temporary folder and compares every output with the
//           committed file byte for byte; exit 1 names each file that differs.
// --help    prints this text.
//
// Outputs (all committed):
//   app/macos/Runner/Assets.xcassets/AppIcon.appiconset/  app_icon_{16..1024}.png
//                                                          and Contents.json
//   app/windows/runner/resources/app_icon.ico              PNG layers 16, 24, 32,
//                                                          48, 64, 128, 256
//   app/linux/runner/resources/app_icon.png                512 px
//   brand/logo.png                                         512 px, for places
//                                                          that show no SVG
//
// The renderer is the resvg CLI (Homebrew: `brew install resvg`, MPL-2.0), pinned
// to RESVG_VERSION: another version may give other bytes, so the script refuses
// it. resvg writes the same PNG bytes for the same SVG. The ico container is
// written here (header, directory, PNG entries in size order), so it holds no
// timestamp and no tool-specific order. macOS gets no icns from this script:
// Xcode compiles the appiconset into the app at build time.
//
// Exit codes: 0 done or identical, 1 a file differs or a render failed, 2 usage
// error or missing or wrong resvg.

import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { parseArgs } from 'node:util'

const ROOT = resolve(import.meta.dir, '..')
const RESVG_VERSION = '0.50.0'
const APPICONSET = 'app/macos/Runner/Assets.xcassets/AppIcon.appiconset'
const MAC_SIZES = [16, 32, 64, 128, 256, 512, 1024]
const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

/** The Contents.json of the appiconset: each point size at 1x and 2x. */
export function contentsJson(): string {
  const images = [16, 32, 128, 256, 512].flatMap((size) =>
    [1, 2].map((scale) => ({
      size: `${size}x${size}`,
      idiom: 'mac',
      filename: `app_icon_${size * scale}.png`,
      scale: `${scale}x`,
    })),
  )
  // Xcode style: `"key" : value`, so an Xcode save leaves the file unchanged.
  const json = JSON.stringify({ images, info: { version: 1, author: 'xcode' } }, null, 2)
  return `${json.replace(/^(\s*"[^"]+"):/gm, '$1 :')}\n`
}

/** An ico file with one PNG entry per image, in the given order. */
export function icoOf(pngs: { size: number; bytes: Buffer }[]): Buffer {
  const header = Buffer.alloc(6)
  header.writeUInt16LE(0, 0) // reserved
  header.writeUInt16LE(1, 2) // type: icon
  header.writeUInt16LE(pngs.length, 4)
  const entries: Buffer[] = []
  let offset = 6 + 16 * pngs.length
  for (const { size, bytes } of pngs) {
    const entry = Buffer.alloc(16)
    entry.writeUInt8(size >= 256 ? 0 : size, 0) // width, 0 means 256
    entry.writeUInt8(size >= 256 ? 0 : size, 1) // height
    entry.writeUInt8(0, 2) // no palette
    entry.writeUInt8(0, 3) // reserved
    entry.writeUInt16LE(1, 4) // color planes
    entry.writeUInt16LE(32, 6) // bits per pixel
    entry.writeUInt32LE(bytes.length, 8)
    entry.writeUInt32LE(offset, 12)
    entries.push(entry)
    offset += bytes.length
  }
  return Buffer.concat([header, ...entries, ...pngs.map((p) => p.bytes)])
}

function resvgVersion(): string | undefined {
  const run = spawnSync('resvg', ['--version'], { encoding: 'utf8' })
  return run.status === 0 ? run.stdout.trim() : undefined
}

function renderPng(source: string, size: number, scratch: string): Buffer {
  const out = join(scratch, `render-${size}.png`)
  const run = spawnSync('resvg', ['-w', String(size), '-h', String(size), source, out], { encoding: 'utf8' })
  if (run.status !== 0) throw new Error(`resvg ${size} px failed:\n${run.stdout}${run.stderr}`)
  return readFileSync(out)
}

/** Every output file, relative to the repo root, with its bytes. */
function render(source: string, scratch: string): Map<string, Buffer> {
  const sizes = [...new Set([...MAC_SIZES, ...ICO_SIZES, 512])].sort((a, b) => a - b)
  const png = new Map(sizes.map((size) => [size, renderPng(source, size, scratch)]))
  const files = new Map<string, Buffer>()
  for (const size of MAC_SIZES) files.set(`${APPICONSET}/app_icon_${size}.png`, png.get(size)!)
  files.set(`${APPICONSET}/Contents.json`, Buffer.from(contentsJson()))
  files.set(
    'app/windows/runner/resources/app_icon.ico',
    icoOf(ICO_SIZES.map((size) => ({ size, bytes: png.get(size)! }))),
  )
  files.set('app/linux/runner/resources/app_icon.png', png.get(512)!)
  files.set('brand/logo.png', png.get(512)!)
  return files
}

function main(): number {
  const { values } = parseArgs({
    options: {
      source: { type: 'string', default: join(ROOT, 'brand', 'logo.svg') },
      check: { type: 'boolean', default: false },
      help: { type: 'boolean', default: false },
    },
  })
  if (values.help) {
    const head = readFileSync(import.meta.path, 'utf8').split('\n\n')[0]
    console.log(head.replace(/^#!.*\n/, '').replace(/^\/\/ ?/gm, ''))
    return 0
  }
  const found = resvgVersion()
  if (found === undefined) {
    console.error('icon-render: resvg is missing; install it with `brew install resvg`')
    return 2
  }
  if (found !== RESVG_VERSION) {
    console.error(`icon-render: resvg ${found} found, ${RESVG_VERSION} expected (RESVG_VERSION in this script)`)
    return 2
  }
  const source = resolve(values.source)
  if (!existsSync(source)) {
    console.error(`icon-render: no SVG at ${source}`)
    return 2
  }
  const scratch = mkdtempSync(join(tmpdir(), 'ftl-editor-icons-'))
  try {
    const files = render(source, scratch)
    if (values.check) {
      const differ = [...files].filter(([path, bytes]) => {
        const kept = join(ROOT, path)
        return !existsSync(kept) || !readFileSync(kept).equals(bytes)
      })
      for (const [path] of differ) console.error(`differs: ${path}`)
      console.log(`${files.size - differ.length} of ${files.size} files are identical`)
      return differ.length === 0 ? 0 : 1
    }
    for (const [path, bytes] of files) {
      mkdirSync(dirname(join(ROOT, path)), { recursive: true })
      writeFileSync(join(ROOT, path), bytes)
    }
    console.log(`rendered ${files.size} files from ${source}`)
    return 0
  } finally {
    rmSync(scratch, { recursive: true, force: true })
  }
}

if (import.meta.main) process.exit(main())
