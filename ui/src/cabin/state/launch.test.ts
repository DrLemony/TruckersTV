import { describe, expect, it } from 'vitest'
import { carriesLaunch, type Launch, launchFor, transitionKind } from './launch'
import type { Route } from './navigation'

const home: Route = { kind: 'home' }
const spotify: Route = { kind: 'app', app: 'spotify' }
const youtube: Route = { kind: 'app', app: 'youtube' }

describe('launchFor', () => {
  it('opens an app out of the icon that was clicked', () => {
    expect(launchFor(home, spotify, 'home')).toEqual({ app: 'spotify', origin: 'home', direction: 'open' })
    expect(launchFor(youtube, spotify, 'home')).toEqual({ app: 'spotify', origin: 'home', direction: 'open' })
  })

  it('closes an app into its Home icon', () => {
    expect(launchFor(spotify, home, null)).toEqual({ app: 'spotify', origin: 'home', direction: 'close' })
  })

  it('crossfades everything else', () => {
    // An app opened without a clicked icon (no origin) has nothing to grow out of.
    expect(launchFor(home, spotify, null)).toBeNull()
    expect(launchFor(spotify, spotify, 'home')).toBeNull()
  })
})

describe('carriesLaunch', () => {
  const open: Launch = { app: 'spotify', origin: 'home', direction: 'open' }
  const close: Launch = { app: 'spotify', origin: 'home', direction: 'close' }
  const icon = { kind: 'icon', origin: 'home' } as const
  const window = { kind: 'window' } as const

  it('opening: the icon before the navigation, the window after', () => {
    expect(carriesLaunch({ launch: open, phase: 'from' }, 'spotify', icon)).toBe(true)
    expect(carriesLaunch({ launch: open, phase: 'from' }, 'spotify', window)).toBe(false)
    expect(carriesLaunch({ launch: open, phase: 'to' }, 'spotify', icon)).toBe(false)
    expect(carriesLaunch({ launch: open, phase: 'to' }, 'spotify', window)).toBe(true)
  })

  it('closing: the window before, the Home icon after', () => {
    expect(carriesLaunch({ launch: close, phase: 'from' }, 'spotify', window)).toBe(true)
    expect(carriesLaunch({ launch: close, phase: 'to' }, 'spotify', icon)).toBe(true)
    expect(carriesLaunch({ launch: close, phase: 'to' }, 'spotify', window)).toBe(false)
  })

  it('only the launching app takes part', () => {
    expect(carriesLaunch({ launch: open, phase: 'from' }, 'youtube', icon)).toBe(false)
    expect(carriesLaunch({ launch: null, phase: 'from' }, 'spotify', icon)).toBe(false)
  })
})

describe('transitionKind', () => {
  it('opens and closes apps, and crossfades the rest', () => {
    expect(transitionKind({ app: 'spotify', origin: 'home', direction: 'open' })).toBe('open')
    expect(transitionKind({ app: 'spotify', origin: 'home', direction: 'close' })).toBe('close')
    expect(transitionKind(null)).toBe('switch')
  })
})
