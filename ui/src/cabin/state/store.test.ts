import { afterEach, describe, expect, it, vi } from 'vitest'

/** A browser's storage holding what an earlier TruckersTV saved (and the page around it, outside TruckersTV). */
function savedStorage(state: unknown, version = 0) {
  vi.stubGlobal('window', {})
  vi.stubGlobal('location', { search: '' })
  const items = new Map([['truckerstv.cabin', JSON.stringify({ state, version })]])
  return {
    getItem: (key: string) => items.get(key) ?? null,
    setItem: (key: string, value: string) => void items.set(key, value),
    removeItem: (key: string) => void items.delete(key),
  }
}

afterEach(() => {
  vi.unstubAllGlobals()
  vi.resetModules()
})

describe('the saved Text Size', () => {
  it('comes back, and what isn’t a size is left out', async () => {
    vi.stubGlobal('localStorage', savedStorage({ textSize: 5 }))
    const { useCabin } = await import('./store')
    const { textScale } = await import('./textSize')
    expect(textScale(useCabin.getState().textSize)).toBe(1.35)
  })

  it('starts on the default when what was kept is no size', async () => {
    vi.stubGlobal('localStorage', savedStorage({ textSize: 'big' }))
    const { useCabin } = await import('./store')
    const { DEFAULT_TEXT_SIZE } = await import('./textSize')
    expect(useCabin.getState().textSize).toBe(DEFAULT_TEXT_SIZE)
  })
})
