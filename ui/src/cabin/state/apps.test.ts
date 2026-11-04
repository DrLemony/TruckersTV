import { describe, expect, it } from 'vitest'
import { APP_ICONS, APP_IDS, isAppId } from './apps'

describe('apps', () => {
  it('knows its own ids and no others', () => {
    for (const app of APP_IDS) expect(isAppId(app)).toBe(true)
    expect(isAppId('maps')).toBe(false)
    expect(isAppId(3)).toBe(false)
  })

  it('has an entry for each app', () => {
    expect(Object.keys(APP_ICONS).sort()).toEqual([...APP_IDS].sort())
  })
})
