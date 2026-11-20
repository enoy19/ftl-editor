#!/usr/bin/env bun
// Scans built artifacts for paths and names of the build machine before they
// are published. A source tree can be clean while a binary carries the
// absolute paths of the build: Rust panic locations name the cargo registry
// and rustup folders, a Dart AOT snapshot names the file URI of the generated
// plugin registrant, C code names __FILE__.
//
//   bun scripts/scan-binaries.ts <path>... [--forbid <text>]... [--forbid-file <file>]
//
// <path>         a file or a folder (walked, symlinks not followed); every
//                regular file is scanned, binary or not
// --forbid       one more text to refuse (repeatable)
// --forbid-file  more texts, one per line; a line `!<text>` allows <text>
//                where it covers a refused text (a public name that contains
//                a private one); empty lines and `#` lines are skipped.
//                Default: scripts/scan-forbid.txt when it exists (a private
//                file of the maintainer, never part of the public tree)
// --help         print this text
//
// Refused texts, case-insensitive, each as single-byte text (what `strings -a`
// sees) and as UTF-16LE (Windows resources and wide strings):
// - the home folder of this machine (os.homedir(), with / and \ separators)
// - the repository root of this script (the checkout the build ran in)
// - the home folder roots "/Users/", "C:\Users\" and "C:/Users/"
// - every --forbid text and every text of the forbid file
//
// Output: per path the files and bytes scanned, then every hit as
// `<file>: <label> at <offset>: <context>`. Exit code 0 without a hit, 1 with
// a hit, 2 for a usage error.

import { existsSync, lstatSync, readFileSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { join, resolve } from 'node:path'
import { parseArgs } from 'node:util'

const ROOT = resolve(import.meta.dir, '..')

type Needle = { label: string; text: string }

const DEFAULT_FORBID_FILE = join(ROOT, 'scripts/scan-forbid.txt')

/** The texts and allowed texts of a forbid file. */
export function readForbidFile(file: string): { forbid: string[]; allow: string[] } {
  const forbid: string[] = []
  const allow: string[] = []
  for (const raw of readFileSync(file, 'utf8').split('\n')) {
    const line = raw.trim()
    if (line === '' || line.startsWith('#')) continue
    if (line.startsWith('!')) allow.push(line.slice(1).toLowerCase())
    else forbid.push(line)
  }
  return { forbid, allow }
}

export function needles(extra: string[], home = homedir(), root = ROOT): Needle[] {
  const list: Needle[] = [
    { label: 'macOS home', text: '/Users/' },
    { label: 'Windows home', text: 'C:\\Users\\' },
    { label: 'Windows home', text: 'C:/Users/' },
  ]
  for (const [label, path] of [
    ['home folder', home],
    ['checkout', root],
  ] as const) {
    const trimmed = path.replace(/[\\/]+$/, '')
    if (trimmed.length < 4) continue
    for (const text of new Set([trimmed, trimmed.replaceAll('\\', '/'), trimmed.replaceAll('/', '\\')])) {
      list.push({ label, text })
    }
  }
  for (const text of extra) list.push({ label: 'forbidden', text })
  return list
}

/** Whether an allowed text covers the hit of `needle` at `at` in `text`. */
function allowed(text: string, at: number, needle: string, allow: string[]): boolean {
  for (const a of allow) {
    for (let i = a.indexOf(needle); i >= 0; i = a.indexOf(needle, i + 1)) {
      if (text.startsWith(a, at - i)) return true
    }
  }
  return false
}

/** `allowed` for a UTF-16LE hit: two bytes per character. */
function allowedWide(text: string, at: number, needle: string, allow: string[]): boolean {
  for (const a of allow) {
    const wide = [...a].map((c) => `${c}\x00`).join('')
    for (let i = a.indexOf(needle); i >= 0; i = a.indexOf(needle, i + 1)) {
      if (text.startsWith(wide, at - 2 * i)) return true
    }
  }
  return false
}

function escape(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/** One regex per needle over the latin1 text of a file: plain or UTF-16LE. */
function pattern(n: Needle): RegExp {
  const lower = n.text.toLowerCase()
  const wide = [...lower].map((c) => `${escape(c)}\\x00`).join('')
  return new RegExp(`(${escape(lower)})|${wide}`, 'g')
}

function context(text: string, at: number): string {
  return text
    .slice(Math.max(0, at - 40), at + 80)
    .replace(/\x00/g, '')
    .replace(/[^\x20-\x7e]+/g, ' ')
    .trim()
}

export type Hit = { file: string; label: string; offset: number; context: string }

export function scanFile(file: string, list: Needle[], allow: string[] = []): Hit[] {
  const text = readFileSync(file).toString('latin1').toLowerCase()
  const hits: Hit[] = []
  for (const n of list) {
    for (const m of text.matchAll(pattern(n))) {
      const needle = n.text.toLowerCase()
      if (m[1] !== undefined ? allowed(text, m.index, needle, allow) : allowedWide(text, m.index, needle, allow)) continue
      const around = context(text, m.index)
      hits.push({ file, label: `${n.label} "${n.text}"`, offset: m.index, context: around })
      if (hits.length > 200) return hits
    }
  }
  return hits
}

function* files(path: string): Generator<string> {
  const st = lstatSync(path)
  if (st.isDirectory()) {
    for (const name of readdirSync(path).sort()) yield* files(join(path, name))
  } else if (st.isFile()) {
    yield path
  }
}

const HELP = readFileSync(import.meta.path, 'utf8')
  .split('\n')
  .slice(1)
  .filter((l) => l.startsWith('//'))
  .map((l) => l.replace(/^\/\/ ?/, ''))
  .join('\n')

/** Scans every path; prints the summary and the hits; answers the hit count. */
export function scanPaths(paths: string[], extra: string[] = [], forbidFile?: string): number {
  const file = forbidFile ?? (existsSync(DEFAULT_FORBID_FILE) ? DEFAULT_FORBID_FILE : undefined)
  const { forbid, allow } = file ? readForbidFile(file) : { forbid: [], allow: [] }
  if (file) console.log(`scan-binaries: ${forbid.length} texts and ${allow.length} allowed texts from ${file}`)
  const list = needles([...extra, ...forbid])
  let total = 0
  for (const path of paths) {
    let count = 0
    let bytes = 0
    const hits: Hit[] = []
    for (const file of files(resolve(path))) {
      count++
      bytes += lstatSync(file).size
      hits.push(...scanFile(file, list, allow))
    }
    console.log(`scan-binaries: ${path}: ${count} files, ${bytes} bytes, ${hits.length} hits`)
    for (const h of hits) console.log(`  ${h.file}: ${h.label} at ${h.offset}: ${h.context}`)
    total += hits.length
  }
  return total
}

export function main(argv: string[]): number {
  let values: { forbid?: string[]; 'forbid-file'?: string; help?: boolean }
  let positionals: string[]
  try {
    ;({ values, positionals } = parseArgs({
      args: argv,
      options: {
        forbid: { type: 'string', multiple: true },
        'forbid-file': { type: 'string' },
        help: { type: 'boolean' },
      },
      allowPositionals: true,
    }))
  } catch (e) {
    console.error(`scan-binaries: ${(e as Error).message}\n\n${HELP}`)
    return 2
  }
  if (values.help) {
    console.log(HELP)
    return 0
  }
  if (positionals.length === 0) {
    console.error(`scan-binaries: give at least one path\n\n${HELP}`)
    return 2
  }
  const forbidFile = values['forbid-file']
  if (forbidFile !== undefined && !existsSync(forbidFile)) {
    console.error(`scan-binaries: no forbid file at ${forbidFile}`)
    return 2
  }
  return scanPaths(positionals, values.forbid ?? [], forbidFile) > 0 ? 1 : 0
}

if (import.meta.main) process.exit(main(process.argv.slice(2)))
