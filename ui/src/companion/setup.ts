import { call } from './bridge'

/** An install of the game. Mirrors the app's `setup::GameInfo`. */
export interface GameInfo {
  dir: string
  foundBy: 'steam' | 'installed' | 'chosen'
  version: string | null
  /** TruckersTV has been tried with this version. */
  tested: boolean
  dlcs: number
}

/** Mirrors the app's `setup::SetupState`. */
export interface SetupState {
  /** The onboarding was gone through: the companion opens on its own pages. */
  finished: boolean
  /** The game TruckersTV is set up in, if it is. */
  game: GameInfo | null
  /** TruckersTV was updated since it put its plugin and mod into that game: the new ones go in once it's
   * closed (`FinishUpdate`). */
  updated: boolean
  /** Every install of the game found on the PC. */
  found: GameInfo[]
  gameRunning: boolean
  /** The onboarding's language, a game language code (`tr_tr`): the game's own (or one set to try it out,
   * `WESTPLAY_LANGUAGE`); `null` before the game was ever started: then Windows'. */
  language: string | null
}

/** What a check of the game found. Mirrors the app's `setup::GameCheck`. */
export interface GameCheck {
  plugin: 'missing' | 'current' | 'different'
}

/** Mirrors the app's `setup::PluginOutcome`. */
export type PluginOutcome =
  | { state: 'installed' }
  | { state: 'inUse' }
  | { state: 'needsAdministrator' }
  | { state: 'declined' }
  | { state: 'failed'; error: string }

/** Mirrors the app's `setup::InstallReport`. */
export interface InstallReport {
  plugin: PluginOutcome
  modError: string | null
  /** TruckersTV is set up in this game. */
  done: boolean
}

export const setupState = () => call<SetupState>('setup_state')
export const gameRunning = () => call<boolean>('game_running')
/** Windows' folder picker: the game there, `null` if the player cancelled; rejects for a folder that isn't it. */
export const chooseGameFolder = (title: string) => call<GameInfo | null>('choose_game_folder', { title })
export const checkGame = (dir: string) => call<GameCheck>('check_game', { dir })
export const installIntoGame = (dir: string, administrator: boolean) =>
  call<InstallReport>('install_into_game', { dir, administrator })
export const finishSetup = () => call<void>('finish_setup')

/** What taking TruckersTV out of the game did. Mirrors the app's `setup::UninstallReport`; its `plugin` is
 * `installed` once the plugin is gone (the install's outcomes, the other way round). */
export interface UninstallReport {
  plugin: PluginOutcome
  modError: string | null
  profilesChanged: number
  profilesFailed: number
  /** TruckersTV is out of the game: it can leave the PC next. */
  done: boolean
}

/** Takes TruckersTV out of the game (its profiles' mods, its screens, the plugin), the game closed. */
export const uninstallFromGame = (administrator: boolean) =>
  call<UninstallReport>('uninstall_from_game', { administrator })
/** TruckersTV's data goes while the uninstall shows it going (it can take a while): its engine stops, then the
 * sign-ins, and unless `keep`, the maps, the player's own looks and the settings. */
export const removeData = (keep: boolean) => call<void>('remove_data', { keep })
/** TruckersTV leaves the PC (what was in use until it ends, and the app if it was installed), keeping settings
 * and maps if `keep`, and quits. */
export const leavePc = (keep: boolean) => call<void>('leave_pc', { keep })
