import type { GameCheck, GameInfo, InstallReport, SetupState } from './setup'
import type { HostStatus } from './useHostStatus'
import type { Settings } from './useSettings'

/**
 * The window opened in a plain browser (bridge.ts): the app's answers, made up, so its pages can be seen without
 * the app or the game. `?first`: the first run. `?game=off|on|live`: the game closed, open, or the TV on air.
 * Nothing here is ever in the app.
 */
const query = new URLSearchParams(location.search)
const gameState = query.get('game') ?? 'live'

const GAME: GameInfo = {
  dir: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Euro Truck Simulator 2',
  foundBy: 'steam',
  version: '1.61.1.9',
  tested: true,
  dlcs: 34,
}

const setup: SetupState = {
  finished: !query.has('first'),
  game: query.has('first') ? null : GAME,
  updated: false,
  found: [GAME],
  gameRunning: false,
  language: query.get('language'),
  testTv: { x: 1.751, y: 0.662, z: -0.09, width: 0.54 },
}

const check: GameCheck = { plugin: 'current' }

let settings: Settings = {
  interaction: { key: 0xa4, mode: 'hold', sensitivity: 1 },
  keybinds: {
    volumeDown: { key: 0x31, withInteractionKey: true },
    playPause: { key: 0x32, withInteractionKey: true },
    volumeUp: { key: 0x33, withInteractionKey: true },
    zoom: { key: 0x34, withInteractionKey: true },
  },
  tv: { videos: null, brightness: 1 },
}

const status = (): HostStatus => ({
  webviewVersion: '130.0',
  adapter: 'Preview',
  cabAdapter: null,
  gameAdapter: null,
  framesPerSecond: gameState === 'live' ? 30 : 0,
  droppedTotal: 0,
  gameConnected: gameState !== 'off',
  screenDrawing: gameState === 'live',
  truckSupported: true,
  youtubeMusicSignedIn: true,
  youtubeMusicSigningIn: false,
  siteSigningIn: null,
  siteSignedIn: ['netflix'],
  spotifySignedIn: false,
  spotifySigningIn: false,
  error: null,
})

const INSTALLED: InstallReport = { plugin: { state: 'installed' }, modError: null, done: true }

export async function answer(command: string, args?: Record<string, unknown>): Promise<unknown> {
  switch (command) {
    case 'setup_state':
      return setup
    case 'game_running':
      return gameState !== 'off'
    case 'check_game':
      return check
    case 'get_settings':
      return settings
    case 'update_settings':
      settings = args?.settings as Settings
      return settings
    case 'install_into_game':
      return INSTALLED
    case 'finish_setup':
      setup.finished = true
      setup.game = GAME
      return null
    case 'set_test_tv':
      setup.testTv = args?.test as SetupState['testTv']
      return gameState === 'off'
    case 'key_name':
      return `Key ${String(args?.key)}`
    default:
      return null
  }
}

export function subscribe(event: string, handler: (payload: unknown) => void): () => void {
  if (event !== 'host-status') return () => {}
  handler(status())
  const timer = window.setInterval(() => handler(status()), 1000)
  return () => window.clearInterval(timer)
}
