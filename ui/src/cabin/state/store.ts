import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import {
  type Equalizer,
  FLAT,
  type HostMessage,
  hasTruck,
  type JobInfo,
  type KeybindAction,
  isInHost,
  postToHost,
  subscribeHostMessages,
  type Telemetry,
  toEqualizer,
  type TruckInfo,
} from '../../shared/host'
import {
  receiveAnswer as receiveSpotifyAnswer,
  receivePlayback,
  receiveState as receiveSpotifyState,
} from '../apps/spotify/spotify'
import {
  receiveAnswer as receiveYouTubeMusicAnswer,
  receivePlayback as receiveYouTubeMusicPlayback,
  receiveState as receiveYouTubeMusicState,
} from '../apps/youtubeMusic/youtubeMusic'
import { useKeyboard } from '../keyboard/store'
import { isAppId } from './apps'
import { afterRunningTransition, animateNavigation, type LaunchOrigin, launchFor } from './launch'
import {
  INITIAL_NAVIGATION,
  navigate,
  type NavigationAction,
  type NavigationState,
  type Route,
  routeKey,
} from './navigation'
import { INITIAL_POWER, power, type PowerEvent, type PowerState } from './power'
import { DEFAULT_TEXT_SIZE, toTextSize } from './textSize'

/**
 * TV state that the UI renders from. Telemetry lives in its own store (`useLive`) because it updates up to 20
 * times a second; components subscribe to just the fields they need.
 */
interface CabinState {
  navigation: NavigationState
  /** The apps' text size, a step of `TEXT_SIZES` (Settings). */
  textSize: number
  /** The player's own tone curve (Settings), and whether it plays (Custom) or the speakers as tuned
   * (Recommended, flat): kept with the app's settings, the host sends them. */
  equalizer: Equalizer
  equalizerCustom: boolean
  power: PowerState
  navigate(action: NavigationAction): void
  setTextSize(step: number): void
  /** One band of the curve (choosing Custom): played by every player, and kept with the app's settings. */
  setEqualizerBand(band: keyof Equalizer, db: number): void
  /** Custom (the curve plays) or Recommended (flat), the curve kept either way. */
  chooseEqualizer(custom: boolean): void
  /** The curve flat again. */
  resetEqualizer(): void
  powerEvent(event: PowerEvent): void
}

/** What survives a restart of the app: where you were and your choices, never the power state. */
type Persisted = Pick<CabinState, 'navigation' | 'textSize'>

export const useCabin = create<CabinState>()(
  persist(
    (set) => ({
      navigation: INITIAL_NAVIGATION,
      textSize: DEFAULT_TEXT_SIZE,
      equalizer: FLAT,
      equalizerCustom: false,
      power: INITIAL_POWER,
      navigate: (action) => set((state) => ({ navigation: navigate(state.navigation, action) })),
      setTextSize: (step) => set({ textSize: toTextSize(step) }),
      setEqualizerBand: (band, db) => {
        const equalizer = toEqualizer({ ...useCabin.getState().equalizer, [band]: db })
        set({ equalizer, equalizerCustom: true })
        if (isInHost()) postToHost({ type: 'equalizer', custom: true, ...equalizer })
      },
      chooseEqualizer: (custom) => {
        set({ equalizerCustom: custom })
        if (isInHost()) postToHost({ type: 'equalizer', custom, ...useCabin.getState().equalizer })
      },
      resetEqualizer: () => {
        set({ equalizer: FLAT })
        if (isInHost()) postToHost({ type: 'equalizer', custom: useCabin.getState().equalizerCustom, ...FLAT })
      },
      powerEvent: (event) => set((state) => ({ power: power(state.power, event) })),
    }),
    {
      name: 'truckerstv.cabin',
      storage: createJSONStorage(() => localStorage),
      partialize: (state): Persisted => ({ navigation: state.navigation, textSize: state.textSize }),
      // Stored data may be from an earlier version (or edited): keep only what is valid.
      merge: (stored, current) => ({ ...current, ...sanitize(stored as Partial<Persisted> | undefined) }),
    },
  ),
)

function sanitize(stored: Partial<Persisted> | undefined): Partial<Persisted> {
  if (!stored) return {}
  const result: Partial<Persisted> = {}
  if (stored.navigation && isRoute(stored.navigation.route) && Array.isArray(stored.navigation.recents)) {
    const { route, recents } = stored.navigation
    result.navigation = { route, recents: recents.filter(isAppId) }
  }
  if (stored.textSize !== undefined) result.textSize = toTextSize(stored.textSize)
  return result
}

function isRoute(route: unknown): route is Route {
  if (typeof route !== 'object' || route === null) return false
  const { kind, app } = route as { kind?: unknown; app?: unknown }
  return kind === 'home' || (kind === 'app' && isAppId(app))
}

/** Live data from the game. */
export interface LiveState {
  telemetry: Telemetry | null
  truck: TruckInfo | null
  /** The player's job (`null`: none). */
  job: JobInfo | null
  /** The game's language (its `g_lang_init`, e.g. `tr_tr`; `null` until the host has read it). */
  gameLanguage: string | null
  /** Whether the game's window shows, in front or not (no game: as if it did). */
  gameShown: boolean
  /** The game's menu gone while it's still paused (the music comes back then, not 1.5 s later). */
  menuGone: boolean
  /** The keybind last pressed in the game, and how many so far (each press a new one). */
  keybind: { action: KeybindAction; count: number } | null
  /** The music's spectrum as the host last said it (Settings' equalizer). */
  spectrum: readonly number[]
  /** The host draws the screen's cursor (`cursor` message): the page draws none of its own then. */
  cursorByHost: boolean
}

export const useLive = create<LiveState>()(() => ({
  telemetry: null,
  truck: null,
  job: null,
  gameLanguage: null,
  gameShown: true,
  menuGone: false,
  keybind: null,
  spectrum: [],
  cursorByHost: false,
}))

/**
 * Feeds host messages into the stores, and tells the host the TV is open. Returns the unsubscribe function; call
 * once at startup. The power follows the truck's electrics (and only changes when they change).
 */
export function connectHost(now: () => number = () => performance.now()): () => void {
  const stopMessages = subscribeHostMessages(receiveHostMessage(now))
  // The field selected, for the player's own keyboard to type into (the host passes it to the game plugin).
  const reportSelection = () => postToHost({ type: 'typing', selection: useKeyboard.getState().selection })
  reportSelection()
  const stopSelections = useKeyboard.subscribe((state, previous) => {
    if (state.selection !== previous.selection) reportSelection()
  })
  return () => {
    stopMessages()
    stopSelections()
  }
}

function receiveHostMessage(now: () => number): (message: HostMessage) => void {
  return (message) => {
    switch (message.type) {
      case 'telemetry': {
        const previous = useLive.getState().telemetry
        useLive.setState({ telemetry: message.data })
        if (previous?.electricEnabled !== message.data.electricEnabled) {
          useCabin.getState().powerEvent({ type: 'electrics', on: message.data.electricEnabled, at: now() })
        }
        break
      }
      case 'truck':
        useLive.setState({ truck: hasTruck(message.data) ? message.data : null })
        break
      case 'job':
        useLive.setState({ job: message.data.active ? message.data : null })
        break
      case 'typed':
        for (const event of message.events) useKeyboard.getState().typed(event)
        break
      case 'typingState':
        useKeyboard.setState({ physical: message.state })
        break
      case 'gameLanguage':
        useLive.setState({ gameLanguage: message.language })
        break
      case 'settings':
        useCabin.setState({
          equalizer: toEqualizer(message.equalizer),
          equalizerCustom: message.equalizerCustom === true,
        })
        break
      case 'spectrum':
        useLive.setState({ spectrum: message.bars })
        break
      case 'cursor':
        useLive.setState({ cursorByHost: message.host })
        break
      case 'gameShown':
        useLive.setState({ gameShown: message.shown })
        break
      case 'menuGone':
        useLive.setState({ menuGone: message.gone })
        break
      case 'keybind':
        useLive.setState((live) => ({ keybind: { action: message.action, count: (live.keybind?.count ?? 0) + 1 } }))
        break
      case 'game':
        // The game closed: the TV goes off as with the truck's electrics, and the next game's first telemetry
        // powers it on afresh.
        if (!message.running) {
          useLive.setState({ telemetry: null })
          useCabin.getState().powerEvent({ type: 'electrics', on: false, at: now() })
        }
        break
      case 'spotify':
        receiveSpotifyState(message.signedIn, message.playing, message.signingIn)
        break
      case 'spotifyAnswer':
        receiveSpotifyAnswer(message)
        break
      case 'spotifyPlayback':
        receivePlayback(message.playback)
        break
      case 'youtubeMusic':
        receiveYouTubeMusicState(message)
        break
      case 'youtubeMusicAnswer':
        receiveYouTubeMusicAnswer(message)
        break
      case 'youtubeMusicPlayback':
        receiveYouTubeMusicPlayback(message.playback)
        break
    }
  }
}

let latestNavigation = 0

/** Goes where a press goes. */
export function go(action: NavigationAction, origin: LaunchOrigin | null = null): void {
  const id = ++latestNavigation
  afterRunningTransition(() => {
    const cabin = useCabin.getState()
    const before = cabin.navigation.route
    const after = navigate(cabin.navigation, action).route
    // Overtaken by a later press, or nothing to change: no View Transition.
    if (id !== latestNavigation || routeKey(after) === routeKey(before)) {
      cabin.navigate(action)
      return
    }
    animateNavigation(launchFor(before, after, origin), () => useCabin.getState().navigate(action))
  })
}
