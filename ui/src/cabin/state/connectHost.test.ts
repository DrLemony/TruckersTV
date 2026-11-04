import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { HostMessage, PageMessage, Telemetry } from '../../shared/host'

/** A stand-in for WebView2's `chrome.webview`: what the page posts, and a way to send it host messages. */
function fakeWebView() {
  const posted: PageMessage[] = []
  const listeners = new Set<(event: { data: HostMessage }) => void>()
  return {
    posted,
    send: (data: HostMessage) => listeners.forEach((listener) => listener({ data })),
    bridge: {
      addEventListener: (_: 'message', listener: (event: { data: HostMessage }) => void) => listeners.add(listener),
      removeEventListener: (_: 'message', listener: (event: { data: HostMessage }) => void) =>
        listeners.delete(listener),
      postMessage: (message: PageMessage) => posted.push(message),
    },
  }
}

beforeEach(() => {
  vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => {}, removeItem: () => {} })
})

afterEach(() => {
  vi.unstubAllGlobals()
  vi.resetModules()
})

describe('connectHost', () => {
  it('tells the host it is ready as it connects', async () => {
    const webview = fakeWebView()
    vi.stubGlobal('window', { chrome: { webview: webview.bridge } })
    const { connectHost } = await import('./store')
    const disconnect = connectHost(() => 0)
    // So the host sends its state again.
    expect(webview.posted[0]).toEqual({ type: 'ready' })
    disconnect()
  })

  it('keeps the tone the host has, and hands each band set in Settings to it', async () => {
    const webview = fakeWebView()
    vi.stubGlobal('window', { chrome: { webview: webview.bridge } })
    const { connectHost, useCabin } = await import('./store')
    const disconnect = connectHost(() => 0)
    expect(useCabin.getState().equalizer).toEqual({ bass: 0, mid: 0, treble: 0 })

    // As the host keeps it; whatever it sends, each band whole dB within ±6 (none said: flat).
    webview.send({ type: 'settings', equalizer: { bass: -3, mid: 2.4, treble: 40 } })
    expect(useCabin.getState().equalizer).toEqual({ bass: -3, mid: 2, treble: 6 })
    webview.send({ type: 'settings' })
    expect(useCabin.getState().equalizer).toEqual({ bass: 0, mid: 0, treble: 0 })

    expect(useCabin.getState().equalizerCustom).toBe(false)

    // A band set: Custom, and the whole curve to the host, to keep and play.
    useCabin.getState().setEqualizerBand('bass', -4)
    expect(useCabin.getState().equalizer).toEqual({ bass: -4, mid: 0, treble: 0 })
    expect(useCabin.getState().equalizerCustom).toBe(true)
    expect(webview.posted.at(-1)).toEqual({ type: 'equalizer', custom: true, bass: -4, mid: 0, treble: 0 })
    useCabin.getState().setEqualizerBand('treble', 9)
    expect(webview.posted.at(-1)).toEqual({ type: 'equalizer', custom: true, bass: -4, mid: 0, treble: 6 })
    // Recommended: the curve kept, flat played.
    useCabin.getState().chooseEqualizer(false)
    expect(webview.posted.at(-1)).toEqual({ type: 'equalizer', custom: false, bass: -4, mid: 0, treble: 6 })
    webview.send({
      type: 'settings',
      equalizer: { bass: -4, mid: 0, treble: 6 },
      equalizerCustom: true,
    })
    expect(useCabin.getState().equalizerCustom).toBe(true)
    useCabin.getState().resetEqualizer()
    expect(webview.posted.at(-1)).toEqual({ type: 'equalizer', custom: true, bass: 0, mid: 0, treble: 0 })
    disconnect()
  })

  it('powers off and forgets the telemetry as the game closes, and powers on again with the next game', async () => {
    const webview = fakeWebView()
    vi.stubGlobal('window', { chrome: { webview: webview.bridge } })
    const { connectHost, useCabin, useLive } = await import('./store')
    const disconnect = connectHost(() => 0)
    const electrics = (on: boolean) => ({ electricEnabled: on, paused: false }) as Telemetry
    webview.send({ type: 'telemetry', data: electrics(true) })
    expect(useCabin.getState().power.phase).not.toBe('off')

    // Closed, its last telemetry still saying the electrics are on: off, and nothing kept of it.
    webview.send({ type: 'game', running: false })
    expect(useCabin.getState().power.phase).toBe('off')
    expect(useLive.getState().telemetry).toBeNull()

    // The next game, electrics on as the last one left them: on again.
    webview.send({ type: 'game', running: true })
    webview.send({ type: 'telemetry', data: electrics(true) })
    expect(useCabin.getState().power.phase).not.toBe('off')
    disconnect()
  })

  it("tells the host the field selected, and types what the player's keyboard typed", async () => {
    const webview = fakeWebView()
    vi.stubGlobal('window', { chrome: { webview: webview.bridge } })
    const { connectHost } = await import('./store')
    const { useKeyboard } = await import('../keyboard/store')
    const selections = () => webview.posted.filter((message) => message.type === 'typing')
    const disconnect = connectHost(() => 0)
    expect(selections()).toEqual([{ type: 'typing', selection: 0 }])
    let text = ''
    const field = {
      id: 'search',
      returnKind: 'search' as const,
      read: () => text,
      write: (t: string) => (text = t),
      submit: () => {},
    }
    useKeyboard.getState().attach(field, 'en')
    expect(selections().at(-1)).toEqual({ type: 'typing', selection: useKeyboard.getState().selection })
    webview.send({ type: 'typed', events: [{ text: 'Bari' }, { key: 'backspace' }] })
    expect(text).toBe('Bar')
    webview.send({ type: 'typingState', state: 'paused' })
    expect(useKeyboard.getState().physical).toBe('paused')
    disconnect()
  })
})
