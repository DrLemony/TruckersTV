import { formatGameTime } from '../../shared/format'
import { cabClock } from '../../shared/host'
import { browserCommand, setSiteFull, useSiteFull, useSiteSignIn } from '../apps/chrome/siteView'
import { isWebAppId } from '../apps/store/webApps'
import { useTv } from '../apps/tv/player'
import { go, useCabin, useLive } from '../state/store'
import { useText } from '../state/text'

/** The bar over an open app, the TV's way Home: the mouse's stand-in for a remote's Home button. */
export function TvBar() {
  const route = useCabin((state) => state.navigation.route)
  const text = useText()
  const gameTime = useLive((state) => (state.telemetry ? cabClock(state.telemetry) : null))
  const fullScreen = useTv((state) => state.full)
  const signingIn = useSiteSignIn((state) => state.open)
  const signedIn = useSiteSignIn((state) => state.signedIn)
  const siteFull = useSiteFull((state) => state.full)
  if (route.kind !== 'app' || fullScreen || siteFull) return null
  return (
    <header className="tv-bar" style={{ viewTransitionName: 'tv-bar' }}>
      <button type="button" className="tv-bar-home" onClick={() => go({ type: 'pressHome' })}>
        <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden>
          <path fill="currentColor" d="M12 3 2.5 11.5H5V20a1 1 0 0 0 1 1h4v-6h4v6h4a1 1 0 0 0 1-1v-8.5h2.5z" />
        </svg>
        {text.launcher.home}
      </button>
      <span className="tv-bar-name">{text.apps[route.app]}</span>
      {isWebAppId(route.app) &&
        (signedIn.includes(route.app) ? (
          <button type="button" className="tv-bar-home" onClick={() => setSiteFull(true)}>
            ⛶ {text.tv.fullScreen}
          </button>
        ) : (
          <button
            type="button"
            className="tv-bar-home"
            onClick={() => browserCommand({ action: 'signIn', app: route.app })}
          >
            {signingIn ? text.launcher.signingIn : text.launcher.signIn}
          </button>
        ))}
      <span className="tv-bar-time">{gameTime === null ? '--:--' : formatGameTime(gameTime)}</span>
    </header>
  )
}
