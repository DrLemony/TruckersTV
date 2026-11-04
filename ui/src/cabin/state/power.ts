/**
 * The TV's power, driven by the truck's electrics:
 *
 *   off ──electrics on──▶ starting ──STARTING_MS──▶ on
 *    ▲                                              │
 *    └──────────────── electrics off ◀──────────────┘
 *
 * The game hides the TV while the electrics are off, so `off` is mostly bookkeeping.
 */
export type PowerPhase = 'off' | 'starting' | 'on'

export interface PowerState {
  phase: PowerPhase
  /** When the current phase started (ms, from the caller's clock). */
  since: number
}

export type PowerEvent = { type: 'electrics'; on: boolean; at: number } | { type: 'tick'; at: number }

export const STARTING_MS = 2000

export const INITIAL_POWER: PowerState = { phase: 'off', since: 0 }

export function power(state: PowerState, event: PowerEvent): PowerState {
  switch (event.type) {
    case 'electrics':
      if (!event.on) return state.phase === 'off' ? state : { phase: 'off', since: event.at }
      return state.phase === 'off' ? { phase: 'starting', since: event.at } : state
    case 'tick': {
      // Against the moment the phase is due, computed the same way, so a tick sent for that moment advances it.
      const due = nextPhaseAt(state)
      return due === null || event.at < due ? state : { phase: 'on', since: event.at }
    }
  }
}

/** When (on the caller's clock) the current phase advances by itself, or `null` if it doesn't. */
export function nextPhaseAt(state: PowerState): number | null {
  return state.phase === 'starting' ? state.since + STARTING_MS : null
}
