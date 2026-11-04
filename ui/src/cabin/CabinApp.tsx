import { useEffect } from 'react'
import { isInHost, postToHost } from '../shared/host'
import { keybinds } from './apps/keybinds'
import { oneAtATime } from './apps/oneAtATime'
import { pauseSpotifyWithGame, spotifyPlayer } from './apps/spotify/pauseWithGame'
import { pauseTvWithGame, tvPlayer } from './apps/tv/player'
import { pauseYouTubeMusicWithGame, youtubeMusicPlayer } from './apps/youtubeMusic/pauseWithGame'
import { AppScreen } from './apps/AppScreen'
import { BootScreen } from './boot/BootScreen'
import { TvHome } from './home/TvHome'
import { routeClicksDuringTransitions } from './shell/clickThrough'
import { installDragScroll } from './shell/dragScroll'
import { Cursor } from './shell/Cursor'
import { Shell } from './shell/Shell'
import { showVolume } from './shell/volumeShown'
import { routeKey } from './state/navigation'
import { nextPhaseAt } from './state/power'
import { meterFrames } from './state/frameMeter'
import { connectHost, useCabin, useLive } from './state/store'
import { languageOf } from './state/strings'

/** Plain browser previews have no truck to power the TV; `?power` simulates it there too. */
const FOLLOW_POWER = isInHost() || new URLSearchParams(location.search).has('power')

/** The TV: the shell around the screen for the current route, after it starts. */
export function CabinApp() {
  const route = useCabin((state) => state.navigation.route)
  const phase = useCabin((state) => state.power.phase)
  useEffect(() => connectHost(), [])
  // Spotify, YouTube Music and TV pause with the game (its menu, tabbing out) and play on after.
  useEffect(() => pauseSpotifyWithGame(), [])
  useEffect(() => pauseYouTubeMusicWithGame(), [])
  useEffect(() => pauseTvWithGame(), [])
  // One of them at a time: the newest to start plays on, the others pause (apps/oneAtATime.ts).
  useEffect(() => oneAtATime([spotifyPlayer, youtubeMusicPlayer, tvPlayer]), [])
  // The player's keybinds pressed in the game: the music's volume, or play/pause (apps/keybinds.ts).
  useEffect(() => followKeybinds(), [])
  // The page's own frames while the game runs, for the host's performance line (state/frameMeter.ts).
  useEffect(() => meterFrames(), [])
  useEffect(() => routeClicksDuringTransitions(), [])
  // Lists follow a drag, as on a touch screen (the cab has a mouse).
  useEffect(() => installDragScroll(), [])
  // The page's language, so text set in capitals follows its rules (Turkish: i to İ, not I).
  const language = languageOf(useLive((state) => state.gameLanguage))
  useEffect(() => {
    document.documentElement.lang = language
  }, [language])
  usePowerClock()
  // For the host's startup log: when each boot screen comes up.
  useEffect(() => postToHost({ type: 'startup', stage: `power ${phase}` }), [phase])

  const powered = !FOLLOW_POWER || phase === 'on'

  return (
    <Shell
      overlay={
        <>
          {FOLLOW_POWER && phase !== 'on' && <BootScreen phase={phase} />}
          {/* In a plain browser the system cursor is there already. */}
          {isInHost() && <Cursor />}
        </>
      }
    >
      {powered &&
        (route.kind === 'app' ? <AppScreen key={routeKey(route)} app={route.app} /> : <TvHome key="home" />)}
    </Shell>
  )
}

/** Acts on each keybind pressed in the game, for the cab's players, showing a volume it sets. Returns the
 * function that stops it. */
function followKeybinds(): () => void {
  const binds = keybinds([spotifyPlayer, youtubeMusicPlayer])
  const stop = useLive.subscribe((live, before) => {
    if (!live.keybind || live.keybind.count === before.keybind?.count) return
    const shown = binds.press(live.keybind.action)
    if (shown) showVolume(shown.volume)
  })
  return () => {
    stop()
    binds.stop()
  }
}

/** Advances timed power phases when their time is up. A timer may fire a little early
 * (browsers round its delay down to whole milliseconds), so its tick says the moment it was set for, at the
 * earliest: a tick that didn't advance the phase would leave it there, with no timer left to try again. */
function usePowerClock() {
  const power = useCabin((state) => state.power)
  const powerEvent = useCabin((state) => state.powerEvent)
  useEffect(() => {
    const due = nextPhaseAt(power)
    if (due === null) return
    const wait = Math.ceil(Math.max(0, due - performance.now()))
    const timer = setTimeout(() => powerEvent({ type: 'tick', at: Math.max(performance.now(), due) }), wait)
    return () => clearTimeout(timer)
  }, [power, powerEvent])
}
