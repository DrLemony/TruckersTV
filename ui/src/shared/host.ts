
/**
 * Messages from the TruckersTV host to the cabin page (WebView2 web messages). Mirrors the Rust side:
 * `westplay_shared::Telemetry`, `westplay_shared::TruckInfo`, the screen layout the game draws TruckersTV on
 * (an id from crates/shared/screen_layouts.txt), and the app's dev commands.
 */

export interface Telemetry {
  x: number
  y: number
  z: number
  /** Turns (0..1), 0 = north. */
  heading: number
  pitch: number
  roll: number
  /** m/s, negative when reversing. */
  speed: number
  engineRpm: number
  gear: number
  displayedGear: number
  cruiseControlSpeed: number
  /** Litres. */
  fuel: number
  /** km. */
  fuelRange: number
  /** m / s / m/s (0 = no limit known). */
  navigationDistance: number
  navigationTime: number
  navigationSpeedLimit: number
  /** Minutes since day 0, 00:00 (day 0 is a Monday): the player's own time, which a job's times are in. */
  gameTime: number
  /** In a Convoy session, the session's time less the player's own, in-game minutes (0 out of one): the cab's
   * clock shows `gameTime` plus this ([`cabClock`]). */
  multiplayerTimeOffset: number
  localScale: number
  dashboardBacklight: number
  electricEnabled: boolean
  engineEnabled: boolean
  parkingBrake: boolean
  lowBeam: boolean
  highBeam: boolean
  leftBlinker: boolean
  rightBlinker: boolean
  paused: boolean
}

/** The time the TV's clock shows (minutes since day 0, 00:00): the game's, or in a Convoy session the session's. */
export function cabClock(telemetry: Telemetry): number {
  // A telemetry without it (none sent yet) is out of a session.
  return telemetry.gameTime + (telemetry.multiplayerTimeOffset || 0)
}

export interface TruckInfo {
  /** e.g. "daf" */
  brandId: string
  /** Display brand, localized by the game, e.g. "DAF". */
  brand: string
  /** Model id, e.g. "vehicle.daf.2021". */
  id: string
  /** Display model name, localized by the game, e.g. "XG+". */
  name: string
}

/** The player's job (`westplay_shared::JobInfo`): names localized by the game. */
export interface JobInfo {
  /** There is a job. */
  active: boolean
  /** The cargo is on: delivering it (otherwise on the way to pick it up). */
  cargoLoaded: boolean
  /** A special transport (no companies). */
  special: boolean
  cargo: string
  /** kg. */
  cargoMass: number
  sourceCity: string
  sourceCompany: string
  destinationCity: string
  destinationCompany: string
  /** When the delivery window ends: minutes since day 0, 00:00 (as `Telemetry.gameTime`). */
  deliveryTime: number
  plannedDistanceKm: number
  income: number
}

/** What a keybind does: the music's volume a step up or down, or it paused or played on. */
export type KeybindAction = 'volumeUp' | 'volumeDown' | 'playPause'

export type HostMessage =
  | {
      type: 'browserState'
      url: string
      title: string
      loading: boolean
      back: boolean
      forward: boolean
      error: string | null
    }
  | { type: 'browserFocus'; editable: boolean }
  | { type: 'browserPress' }
  /** The site app's sign-in window on the desktop, open or closed. */
  | { type: 'browserSignIn'; open: boolean; signedIn: string[] }
  /** TV's player (westplay_host::tv_view): how it plays, a fetch's answer (by the `fetch` command's id), and
   * the PC videos (`folder`: one is chosen, in the companion). */
  | { type: 'tvState'; playing: boolean; loading: boolean; failed: boolean }
  | { type: 'tvFetched'; id: number; ok: boolean; text?: string }
  | { type: 'tvVideos'; folder: boolean; videos: { name: string; url: string }[] }
  | { type: 'tvPasted'; text: string }
  | { type: 'telemetry'; data: Telemetry }
  | { type: 'truck'; data: TruckInfo }
  | { type: 'job'; data: JobInfo }
  /** The settings the page applies, sent as it loads and whenever they change. */
  | {
      type: 'settings'
      /** The player's own tone curve (Settings › Equalizer), and whether it plays (Custom) or not (Recommended). */
      equalizer?: Equalizer
      equalizerCustom?: boolean
    }
  /** The music's spectrum as it plays, while the page asks for it (`spectrum`): bars from the lows to the
   * highs, spaced by octaves, each 0 (silent) to 255; all 0 when it stops. */
  | { type: 'spectrum'; bars: number[] }
  /** Whether the host draws the screen's cursor (into the frames the game takes, where the mouse is at once), or
   * the page draws its own (crates/host cursor.rs). */
  | { type: 'cursor'; host: boolean }
  /** What the player typed on their own keyboard, oldest first (a paste comes as its text). */
  | { type: 'typed'; events: TypedEvent[] }
  /** Whether the player's own keyboard types into the field selected. */
  | { type: 'typingState'; state: TypingState }
  /** The game's language. */
  | { type: 'gameLanguage'; language: string | null }
  /** Whether the game's window shows (`false`: minimised, or on another desktop; in front or not: Lossless
   * Scaling's window is in front of it). */
  | { type: 'gameShown'; shown: boolean }
  /** The game's menu gone while it's still paused: its still picture before the truck moves on. */
  | { type: 'menuGone'; gone: boolean }
  /** A keybind of the player's (the companion's Controls), pressed in the game. */
  | { type: 'keybind'; action: KeybindAction }
  /** Whether the game runs. As it closes, the host closes the web players; the page powers off and forgets
   * its last telemetry. */
  | { type: 'game'; running: boolean }
  /** Spotify's web player: whether it's signed in and playing (`null`: not running), and whether its sign-in
   * window is open on the PC's desktop. Nothing else of the account. */
  | { type: 'spotify'; signedIn: boolean | null; playing: boolean | null; signingIn: boolean }
  /** Spotify's answer to the Spotify app's question `id` (`failed`: Spotify didn't answer). */
  | ({ type: 'spotifyAnswer'; id: number; failed?: boolean } & Partial<SpotifyAnswers>)
  /** What Spotify plays (on TruckersTV, or elsewhere on the account through Spotify Connect); `null`: nothing. */
  | { type: 'spotifyPlayback'; playback: SpotifyPlayback | null }
  /** YouTube Music's web player: whether it's signed in and playing (`null`: not running), whether its sign-in
   * window is open on the PC, and the account's picture (Google's). */
  | {
      type: 'youtubeMusic'
      signedIn: boolean | null
      playing: boolean | null
      signingIn: boolean
      avatar: string | null
    }
  /** YouTube Music's answer to the app's question `id` (`failed`: YouTube Music didn't answer). */
  | ({ type: 'youtubeMusicAnswer'; id: number; failed?: boolean } & Partial<YouTubeMusicAnswers>)
  /** What YouTube Music plays; `null`: nothing. */
  | { type: 'youtubeMusicPlayback'; playback: YouTubeMusicPlayback | null }

/** Messages from the cabin page to the host. */
/** The players' tone controls, as a car stereo's: bass, mid and treble, in dB each (`EQUALIZER_MOST` either way;
 * the host's `speakers::Equalizer`). */
export interface Equalizer {
  bass: number
  mid: number
  treble: number
}

export const EQUALIZER_BANDS = ['bass', 'mid', 'treble'] as const satisfies readonly (keyof Equalizer)[]

/** How far each band goes, dB (the host's `Equalizer::MOST`). */
export const EQUALIZER_MOST = 6

export const FLAT: Equalizer = { bass: 0, mid: 0, treble: 0 }

/** The tone as the host may send it: each band a whole number of dB within the range (anything else flat). */
export function toEqualizer(value: unknown): Equalizer {
  const told = (value ?? {}) as Partial<Record<keyof Equalizer, unknown>>
  const band = (db: unknown) =>
    typeof db === 'number' && Number.isFinite(db)
      ? Math.max(-EQUALIZER_MOST, Math.min(EQUALIZER_MOST, Math.round(db)))
      : 0
  return { bass: band(told.bass), mid: band(told.mid), treble: band(told.treble) }
}

export type PageMessage =
  | { type: 'browser'; command: BrowserCommand }
  | { type: 'tv'; command: TvCommand }
  | { type: 'ready' }
  /** A startup stage reached (the boot screens shown), for the host's startup log. */
  | { type: 'startup'; stage: string }
  /** The players' tone, set in Settings › Equalizer: the player's curve, and whether it plays (Custom) or the
   * speakers as tuned (Recommended); kept with TruckersTV's settings and played by every player. */
  | ({ type: 'equalizer'; custom: boolean } & Equalizer)
  /** The music's spectrum wanted (Settings › Equalizer open) or not: the host answers `spectrum` as it plays. */
  | { type: 'spectrum'; on: boolean }
  /** Whether what's under the cursor is light: the host draws it dark there. */
  | { type: 'cursorLight'; light: boolean }
  /** The text field selected, a number that changes with every selection (0: none), for the player's own
   * keyboard to type into while the truck stands still. */
  | { type: 'typing'; selection: number }
  /** A command for Spotify's web player (`westplay_host::spotify::Command`): `show` places it, in the frame's
   * pixels, where the Spotify app cuts the page open. */
  | { type: 'spotify'; command: SpotifyCommand }
  /** A command for YouTube Music's web player (`westplay_host::youtube_music::Command`). */
  | { type: 'youtubeMusic'; command: YouTubeMusicCommand }

/** What the Spotify app asks of Spotify's web player (`westplay_host::spotify::Command`), run hidden by the
 * host: its sign-in, a question (answered with `spotifyAnswer` and the same `id`), and the player's controls
 * (`play` with a context plays it, from `track`; without, it plays on). */
export type BrowserCommand =
  | { action: 'navigate'; url: string }
  | { action: 'back' | 'forward' | 'reload' | 'stop' | 'hide' | 'close' }
  | { action: 'show'; x: number; y: number; width: number; height: number }
  | { action: 'zoom'; factor: number }
  | { action: 'input'; event: TypedEvent }
  | { action: 'signIn'; app: string }

/** What the TV app asks of its player (`westplay_host::tv_view::Command`): where it shows, what it plays,
 * a playlist or Xtream's answer fetched (answered with `tvFetched`), the PC videos (`tvVideos`) and the PC clipboard's text (`tvPasted`). */
export type TvCommand =
  | { action: 'show'; x: number; y: number; width: number; height: number }
  | { action: 'hide' | 'close' | 'pause' | 'resume' | 'stop' | 'videos' | 'paste' }
  | { action: 'play'; url: string }
  | { action: 'volume'; share: number }
  | { action: 'fetch'; id: number; url: string }
  /** The X over the full screen picture, shown or gone. */
  | { action: 'exit'; shown: boolean }

export type SpotifyCommand =
  /** Run (hidden), saying whether it's signed in (the Spotify app opened). */
  | { action: 'start' }
  | { action: 'signIn' }
  | { action: 'signUp' }
  | { action: 'signOut' }
  | { action: 'ask'; id: number; question: SpotifyQuestion }
  | { action: 'play'; context?: string; track?: string }
  | { action: 'pause' }
  | { action: 'next' }
  | { action: 'previous' }
  | { action: 'seek'; seconds: number }
  /** The playing device's volume (Spotify's own), 0 to 100. */
  | { action: 'volume'; percent: number }
  /** The cab's fade, in the page's own player: its volume times `level` (0 to 1; 1, as Spotify has it). */
  | { action: 'fade'; level: number }
  | { action: 'shuffle'; on: boolean }
  | { action: 'repeat'; mode: SpotifyRepeat }
  /** Into Liked Songs or Your Library (`on`), or out of it. */
  | { action: 'save'; uri: string; on: boolean }

export type SpotifyRepeat = 'off' | 'all' | 'one'

/** What a row, a card or a shortcut shows: under its name, a card shows its description (or who it's by), a
 * row what it is and who it's by ("Playlist • Spotify", built in the cab's language). A song as a card (Home's
 * Trending songs) plays from its album (`context`). */
export interface SpotifyItem {
  uri: string
  kind: 'playlist' | 'album' | 'artist' | 'show' | 'liked' | 'track'
  name: string
  /** The playlist's owner, the album's artists, the podcast's publisher, the song's artists ("" for an
   * artist). */
  by: string
  description: string
  /** Liked Songs' count (its row says it in place of an owner). */
  songs?: number
  image: string | null
  /** The cover's own colour (Spotify extracts one for every image): pages and the player are washed in it. */
  color: string
  pinned?: boolean
  downloaded?: boolean
  /** In Your Library (Spotify says, signed in). */
  saved?: boolean
  /** A song's album. */
  context?: string
}

export interface SpotifyArtistName {
  uri: string
  name: string
}

export interface SpotifyTrack {
  uri: string
  name: string
  artists: SpotifyArtistName[]
  album: { uri: string; name: string }
  image: string | null
  color: string
  /** Seconds. */
  duration: number
  explicit: boolean
  /** In Liked Songs (Spotify says, signed in). */
  saved?: boolean
}

/** A playlist's or album's page. */
export interface SpotifyCollection {
  item: SpotifyItem
  description: string
  /** "Spotify", or the album's artist; and an album's year. */
  owner: string
  year: number | null
  tracks: SpotifyTrack[]
  /** Where its songs go on, while there are more than these: asked as the question's `more`. */
  more?: string
}

export interface SpotifyArtist {
  item: SpotifyItem
  listeners: number
  verified: boolean
  popular: SpotifyTrack[]
  releases: SpotifyItem[]
}

/** Home: Spotify's chips (each a facet of Home, its own words), its shortcuts, and its shelves. */
export interface SpotifyHome {
  chips: { id: string; label: string }[]
  shortcuts: SpotifyItem[]
  /** Its shelves; `recent`: Spotify's Recently played. */
  sections: { title: string; items: SpotifyItem[]; recent?: boolean }[]
}

export interface SpotifyCategory {
  name: string
  color: string
  /** A cover leaning in its corner, as Spotify's tiles have. */
  cover: SpotifyItem | null
}

export interface SpotifyResults {
  top: SpotifyItem | null
  /** The top result when it's a song. */
  topTrack: SpotifyTrack | null
  tracks: SpotifyTrack[]
  artists: SpotifyItem[]
  playlists: SpotifyItem[]
  albums: SpotifyItem[]
}

/** What the Spotify app asks Spotify: Home (a facet of it, `''` for all; the player's own signed in,
 * Spotify's public one otherwise), Your Library, Browse all, a search, a playlist's or album's page, an
 * artist's. */
export type SpotifyQuestion =
  | { ask: 'home'; facet: string; signedIn: boolean }
  | { ask: 'lyrics'; uri: string }
  | { ask: 'library' }
  | { ask: 'browse' }
  | { ask: 'search'; query: string }
  | { ask: 'collection'; uri: string; more?: string }
  | { ask: 'artist'; uri: string }

/** Each question's answer, by the question. */
export interface SpotifyAnswers {
  home: SpotifyHome
  library: SpotifyItem[]
  browse: SpotifyCategory[]
  search: SpotifyResults
  collection: SpotifyCollection
  artist: SpotifyArtist
  lyrics: Lyrics
}

/** What Spotify plays: the song, where it plays from, how far (seconds, as the host said it), the options,
 * and the songs to come. */
export interface SpotifyPlayback {
  track: SpotifyTrack | null
  context: { uri: string; name: string; kind: SpotifyItem['kind'] } | null
  playing: boolean
  position: number
  duration: number
  shuffle: boolean
  repeat: SpotifyRepeat
  /** The playing device's volume, 0 to 1 (`null`: none Spotify can set). */
  volume: number | null
  /** Playing on TruckersTV's own device, not another of the account's (a phone); missing: taken as here. */
  here?: boolean
  next: SpotifyTrack[]
}

/** A song's words, as Spotify answers them (`westplay_host::lyrics::Lyrics`): line by line with
 * when each is sung (`synced`; seconds, `end` 0: until the next), where a stanza starts, where they're from
 * (Spotify's provider), Spotify's colours for them, and whether they're written
 * right to left. No lines: no words for the song. */
export interface Lyrics {
  synced: boolean
  lines: { start: number; end: number; text: string; stanza?: boolean }[]
  provider?: string
  writers?: string[]
  colors?: { background: string; text: string; highlight: string }
  rtl?: boolean
}

/** What the YouTube Music app asks of YouTube Music's web player (`westplay_host::youtube_music::Command`), run
 * hidden by the host: its sign-in (Google's), a question (answered with
 * `youtubeMusicAnswer` and the same `id`), the language it answers in, and the player's controls (`play` with
 * `watch` plays it; without, it plays on). */
export type YouTubeMusicCommand =
  | { action: 'start' }
  | { action: 'signIn' }
  | { action: 'signOut' }
  | { action: 'ask'; id: number; question: YouTubeMusicQuestion }
  | { action: 'language'; code: 'en' | 'tr' }
  | { action: 'play'; watch?: YouTubeMusicWatch }
  | { action: 'pause' }
  | { action: 'next' }
  | { action: 'previous' }
  | { action: 'seek'; seconds: number }
  | { action: 'volume'; percent: number }
  | { action: 'shuffle'; on: boolean }
  | { action: 'repeat'; mode: YouTubeMusicRepeat }
  /** The queue's song at `index` (its place in the whole queue). */
  | { action: 'jump'; index: number }
  | { action: 'like'; video: string; like: YouTubeMusicRating }
  /** A song into the queue: right after the one playing (`next`), or at its end. */
  | { action: 'queue'; video: string; next: boolean }
  /** An album or playlist (its playlist id) saved to the Library, or taken out. */
  | { action: 'save'; playlist: string; on: boolean }
  /** The Song/Video switch: the music video's picture (`true`), or the song. */
  | { action: 'video'; on: boolean }
  /** The music video's picture placed at this rectangle of the frame (pixels), under the page; or none. */
  | { action: 'show'; x: number; y: number; width: number; height: number }
  | { action: 'hide' }

export type YouTubeMusicRepeat = 'off' | 'all' | 'one'
export type YouTubeMusicRating = 'like' | 'dislike' | 'none'
export type YouTubeMusicShelf = 'playlists' | 'songs' | 'albums' | 'artists' | 'subscriptions' | 'history'
export type YouTubeMusicKind =
  'song' | 'video' | 'episode' | 'album' | 'playlist' | 'artist' | 'channel' | 'podcast' | 'mood'

/** Playing something: a song (`video`), from a list (`playlist`, at `index`), with YouTube's own `params` (a
 * radio, a shuffle), as YouTube Music's answers give them. */
export interface YouTubeMusicWatch {
  video?: string
  playlist?: string
  params?: string
  index?: number
}

/** A page of YouTube Music's: its id, and its params (a mood's, a chart's). */
export interface YouTubeMusicBrowse {
  id: string
  params?: string
}

/** A card or a row: what it is, its title and YouTube Music's own line under it, its picture, its page,
 * playing it, a song's id, artist and album pages and length, a mood's colour, its place (a number). */
export interface YouTubeMusicItem {
  kind: YouTubeMusicKind
  title: string
  subtitle: string
  image?: string
  browse?: YouTubeMusicBrowse
  play?: YouTubeMusicWatch
  video?: string
  artist?: YouTubeMusicBrowse
  album?: YouTubeMusicBrowse
  duration?: string
  explicit?: boolean
  color?: string
  rank?: string
}

/** Square covers side by side; songs in columns of four (Quick picks); songs one under another; mood
 * buttons, all of them or in rows side by side; words. */
export type YouTubeMusicStyle = 'shelf' | 'columns' | 'songs' | 'moods' | 'moodRows' | 'about'

export interface YouTubeMusicSection {
  title: string
  strapline?: string
  style: YouTubeMusicStyle
  items: YouTubeMusicItem[]
  text?: string
  more?: YouTubeMusicBrowse
}

export interface YouTubeMusicChip {
  label: string
  params?: string
  selected?: boolean
}

/** Home, Explore, a page, a song's Related, the Library's: its choices, shelves and the token for more. */
export interface YouTubeMusicPage {
  title?: string
  /** The picture YouTube Music shows behind its top (Home's, as its mood is). */
  background?: string
  chips: YouTubeMusicChip[]
  sections: YouTubeMusicSection[]
  more?: string
}

export interface YouTubeMusicCollection {
  kind: 'album' | 'playlist'
  title: string
  subtitle: string
  second: string
  by: string
  byPage?: YouTubeMusicBrowse
  image?: string
  description: string
  play?: YouTubeMusicWatch
  shuffle?: YouTubeMusicWatch
  radio?: YouTubeMusicWatch
  playlist?: string
  saved?: boolean
  tracks: YouTubeMusicItem[]
  sections: YouTubeMusicSection[]
  more?: string
}

export interface YouTubeMusicArtist {
  name: string
  description: string
  image?: string
  subscribers: string
  shuffle?: YouTubeMusicWatch
  radio?: YouTubeMusicWatch
  sections: YouTubeMusicSection[]
}

export interface YouTubeMusicResults {
  top?: YouTubeMusicItem
  topSongs: YouTubeMusicItem[]
  chips: YouTubeMusicChip[]
  items: YouTubeMusicItem[]
  more?: string
}

export interface YouTubeMusicSuggestions {
  texts: string[]
  items: YouTubeMusicItem[]
}

export interface YouTubeMusicLyrics {
  text: string
  source: string
}

export type YouTubeMusicQuestion =
  | { ask: 'home'; chip?: string; more?: string }
  | { ask: 'explore' }
  | { ask: 'browse'; page: YouTubeMusicBrowse }
  | { ask: 'search'; query: string; filter?: string; more?: string }
  | { ask: 'suggest'; query: string }
  | { ask: 'collection'; id: string; more?: string }
  | { ask: 'artist'; id: string }
  | { ask: 'library'; shelf: YouTubeMusicShelf }
  | { ask: 'lyrics'; video: string }
  | { ask: 'related'; video: string }

export interface YouTubeMusicAnswers {
  home: YouTubeMusicPage
  explore: YouTubeMusicPage
  browse: YouTubeMusicPage
  search: YouTubeMusicResults
  suggest: YouTubeMusicSuggestions
  collection: YouTubeMusicCollection
  artist: YouTubeMusicArtist
  library: YouTubeMusicPage
  lyrics: YouTubeMusicLyrics
  related: YouTubeMusicPage
}

/** The song playing: its id, title, artists (and their page), album (and its page), cover, length, kind, like,
 * and whether it has a music video to show. */
export interface YouTubeMusicTrack {
  video: string
  title: string
  artists: string
  artist?: YouTubeMusicBrowse
  album: string
  albumPage?: YouTubeMusicBrowse
  image?: string
  duration: number
  kind: YouTubeMusicKind
  like: YouTubeMusicRating
  hasVideo: boolean
}

/** What YouTube Music plays: the song, playing (and waiting for its sound), how far, the options, the volume,
 * the Song/Video switch, the queue around the song (`queue` from the whole queue's `start`; `index`: the
 * song's place in the whole queue), what it plays from, and whether an ad plays. */
export interface YouTubeMusicPlayback {
  track: YouTubeMusicTrack
  playing: boolean
  loading: boolean
  position: number
  duration: number
  shuffle: boolean
  repeat: YouTubeMusicRepeat
  volume: number | null
  video: boolean
  queue: YouTubeMusicItem[]
  start: number
  index: number
  from: string
  ad: boolean
}

/** Text typed, or a key with a meaning of its own. */
export type TypedEvent =
  | { text: string }
  | {
      key:
        | 'backspace'
        | 'wordBackspace'
        | 'enter'
        | 'escape'
        | 'tab'
        | 'delete'
        | 'left'
        | 'right'
        | 'home'
        | 'end'
        | 'selectAll'
    }

/** Whether the player's own keyboard types into the field selected: no field selected (`off`), typing, or
 * the keys the game's (`paused`: the truck moved, or the field was selected while it moved). */
export type TypingState = 'off' | 'typing' | 'paused'

interface WebViewBridge {
  addEventListener(type: 'message', listener: (event: { data: HostMessage }) => void): void
  removeEventListener(type: 'message', listener: (event: { data: HostMessage }) => void): void
  postMessage(message: PageMessage): void
}

declare global {
  interface Window {
    chrome?: { webview?: WebViewBridge }
  }
}

/** Whether the page runs inside the TruckersTV host (and not in a plain browser preview). */
export function isInHost(): boolean {
  return window.chrome?.webview !== undefined
}

/**
 * Calls `listener` for every host message. Returns the unsubscribe function.
 *
 * Subscribing also tells the host the page is ready, so it sends the current state again: anything it
 * sent before this page (re)loaded was lost, and some state (like the truck) is only sent on change.
 */
export function subscribeHostMessages(listener: (message: HostMessage) => void): () => void {
  const webview = window.chrome?.webview
  if (!webview) return () => {}
  const onMessage = (event: { data: HostMessage }) => listener(event.data)
  webview.addEventListener('message', onMessage)
  webview.postMessage({ type: 'ready' })
  return () => webview.removeEventListener('message', onMessage)
}

/** Sends a message to the host (nothing happens in a plain browser preview). */
export function postToHost(message: PageMessage): void {
  window.chrome?.webview?.postMessage(message)
}

/** A truck with no identity (the SDK reports empty strings when there is no truck). */
export function hasTruck(truck: TruckInfo | null): truck is TruckInfo {
  return truck !== null && truck.brandId !== ''
}
