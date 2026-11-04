import type { AppId } from '../state/apps'
import { useLaunchStyle } from '../state/launch'
import './apps.css'
import { ChromeApp } from './chrome/ChromeApp'
import { SettingsApp } from './settings/SettingsApp'
import { SpotifyApp } from './spotify/SpotifyApp'
import { isWebAppId } from './store/webApps'
import { WebAppScreen } from './store/WebAppScreen'
import { TvApp } from './tv/TvApp'
import { YouTubeMusicApp } from './youtubeMusic/YouTubeMusicApp'

/** An open app, on the whole TV under its bar. */
export function AppScreen({ app }: { app: AppId }) {
  const launch = useLaunchStyle(app, { kind: 'window' })
  return (
    <div className="screen app-window" style={launch}>
      <AppContent app={app} />
    </div>
  )
}

function AppContent({ app }: { app: AppId }) {
  if (isWebAppId(app)) return <WebAppScreen app={app} />
  switch (app) {
    case 'chrome':
      return <ChromeApp />
    case 'tv':
      return <TvApp />
    case 'spotify':
      return <SpotifyApp />
    case 'youtubeMusic':
      return <YouTubeMusicApp />
    case 'settings':
      return <SettingsApp />
  }
}
