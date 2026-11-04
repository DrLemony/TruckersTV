import { describe, expect, it } from 'vitest'
import { loadInstalled, WEB_APPS } from './webApps'

function storage(value: string | null): Storage {
  return { getItem: () => value } as unknown as Storage
}

describe('the App Store', () => {
  it('lists each app once, at a web address', () => {
    expect(new Set(WEB_APPS.map((app) => app.id)).size).toBe(WEB_APPS.length)
    for (const app of WEB_APPS) expect(app.url).toMatch(/^https:\/\//)
  })

  it('reads back the apps got, once each, leaving out what the store no longer has', () => {
    expect(loadInstalled(storage('["tod","netflix","tod","gone",3]'))).toEqual(['tod', 'netflix'])
    expect(loadInstalled(storage('not json'))).toEqual([])
    expect(loadInstalled(storage(null))).toEqual([])
  })
})
