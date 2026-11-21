import { describe, expect, test } from 'bun:test'
import { join } from 'node:path'
import { datsDirCandidates, parseJavaMajor, parseMavenJavaMajor, patchState, rememberedDatsDir } from './oracle'

describe('parseJavaMajor', () => {
  test('reads the major of Java 9 and later', () => {
    expect(parseJavaMajor('openjdk version "21.0.10" 2026-01-20 LTS\nOpenJDK Runtime Environment')).toBe(21)
    expect(parseJavaMajor('java version "17" 2021-09-14 LTS')).toBe(17)
    expect(parseJavaMajor('openjdk version "25-ea" 2025-09-16')).toBe(25)
  })
  test('reads 1.x as the old major', () => {
    expect(parseJavaMajor('java version "1.8.0_392"')).toBe(8)
  })
  test('answers undefined for text without a version', () => {
    expect(parseJavaMajor('zsh: command not found: java')).toBeUndefined()
  })
})

describe('parseMavenJavaMajor', () => {
  test('reads the Java version Maven runs on', () => {
    const v = 'Apache Maven 3.9.15 (98b2)\nMaven home: /opt/maven\nJava version: 21.0.10, vendor: Eclipse Adoptium, runtime: /jdk\n'
    expect(parseMavenJavaMajor(v)).toBe(21)
    expect(parseMavenJavaMajor('Java version: 1.8.0_392, vendor: Oracle')).toBe(8)
  })
  test('answers undefined without the line', () => {
    expect(parseMavenJavaMajor('Apache Maven 3.9.15')).toBeUndefined()
  })
})

describe('patchState', () => {
  test('forward applies: unapplied', () => expect(patchState(true, false)).toBe('unapplied'))
  test('only reverse applies: applied', () => expect(patchState(false, true)).toBe('applied'))
  test('neither applies: conflict', () => expect(patchState(false, false)).toBe('conflict'))
  // A patch that only adds files cannot apply forward and reverse at once, but a
  // patch that applies both ways is not yet in the tree: forward wins.
  test('both apply: unapplied', () => expect(patchState(true, true)).toBe('unapplied'))
})

describe('datsDirCandidates', () => {
  test('macOS: the two Steam spellings, then /Applications', () => {
    expect(datsDirCandidates({ os: 'darwin', home: '/Users/someone' })).toEqual([
      '/Users/someone/Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources',
      '/Users/someone/Library/Application Support/Steam/SteamApps/common/FTL Faster Than Light/FTL.app/Contents/Resources',
      '/Applications/FTL.app/Contents/Resources',
    ])
  })
  test('Linux: XDG Steam first, Wine last', () => {
    const dirs = datsDirCandidates({ os: 'linux', home: '/home/x' })
    expect(dirs[0]).toBe(join('/home/x/.local/share/Steam', 'steamapps', 'common/FTL Faster Than Light', 'data'))
    expect(dirs.at(-1)).toBe(join('/home/x/.wine', 'drive_c', 'Program Files', 'FTL', 'resources'))
    expect(dirs).toHaveLength(16)
  })
  test('Windows: Program Files (x86) first', () => {
    const dirs = datsDirCandidates({ os: 'win32', home: 'C:\\Users\\x', programFilesX86: 'D:\\PF86', programFiles: 'D:\\PF' })
    expect(dirs[0]).toBe(join('D:\\PF86', '', 'Steam/steamapps/common/FTL Faster Than Light'))
    expect(dirs).toHaveLength(12)
  })
})

describe('rememberedDatsDir', () => {
  test('the folder of the remembered ftl.dat', () => {
    expect(rememberedDatsDir('{"ftl_dat": "/Games/FTL.app/Contents/Resources/ftl.dat"}')).toBe('/Games/FTL.app/Contents/Resources')
  })
  test('nothing remembered or unreadable JSON: undefined', () => {
    expect(rememberedDatsDir('{}')).toBeUndefined()
    expect(rememberedDatsDir('{"ftl_dat": null}')).toBeUndefined()
    expect(rememberedDatsDir('not json')).toBeUndefined()
  })
})
