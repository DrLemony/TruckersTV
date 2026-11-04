import { describe, expect, it } from 'vitest'
import { INITIAL_NAVIGATION, navigate, RECENTS_LIMIT, routeKey } from './navigation'

describe('navigation', () => {
  it('opens an app, moving it to the front of the recents', () => {
    const first = navigate(INITIAL_NAVIGATION, { type: 'openApp', app: 'spotify' })
    const second = navigate(first, { type: 'openApp', app: 'youtube' })
    const again = navigate(second, { type: 'openApp', app: 'spotify' })
    expect(again.route).toEqual({ kind: 'app', app: 'spotify' })
    expect(again.recents).toEqual(['spotify', 'youtube'])
  })

  it('keeps a limited number of recents', () => {
    const apps = ['youtube', 'spotify', 'youtubeMusic', 'tv', 'chrome', 'netflix'] as const
    const state = apps.reduce((now, app) => navigate(now, { type: 'openApp', app }), INITIAL_NAVIGATION)
    expect(state.recents).toHaveLength(RECENTS_LIMIT)
  })

  it('goes home from an app, the recents kept', () => {
    const open = navigate(INITIAL_NAVIGATION, { type: 'openApp', app: 'tv' })
    const home = navigate(open, { type: 'pressHome' })
    expect(home.route).toEqual({ kind: 'home' })
    expect(home.recents).toEqual(['tv'])
  })

  it('has a key for each route', () => {
    expect(routeKey({ kind: 'home' })).toBe('home')
    expect(routeKey({ kind: 'app', app: 'tv' })).toBe('app:tv')
  })
})
