import { describe, expect, it } from 'vitest'
import { INITIAL_POWER, nextPhaseAt, power, STARTING_MS } from './power'

describe('power', () => {
  it('starts with the electrics and comes on when the time is up', () => {
    const starting = power(INITIAL_POWER, { type: 'electrics', on: true, at: 100 })
    expect(starting).toEqual({ phase: 'starting', since: 100 })
    expect(nextPhaseAt(starting)).toBe(100 + STARTING_MS)
    expect(power(starting, { type: 'tick', at: 100 + STARTING_MS - 1 })).toBe(starting)
    expect(power(starting, { type: 'tick', at: 100 + STARTING_MS }).phase).toBe('on')
  })

  it('goes off with the electrics, from any phase', () => {
    const on = { phase: 'on', since: 0 } as const
    expect(power(on, { type: 'electrics', on: false, at: 5 })).toEqual({ phase: 'off', since: 5 })
    expect(power(INITIAL_POWER, { type: 'electrics', on: false, at: 5 })).toBe(INITIAL_POWER)
  })

  it('takes no notice of the electrics staying on', () => {
    const on = { phase: 'on', since: 0 } as const
    expect(power(on, { type: 'electrics', on: true, at: 5 })).toBe(on)
    expect(nextPhaseAt(on)).toBeNull()
  })
})
