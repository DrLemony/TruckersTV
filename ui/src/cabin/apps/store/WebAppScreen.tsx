import { useEffect, useRef } from 'react'
import { isInHost } from '../../../shared/host'
import { useText } from '../../state/text'
import { setSiteFull, useSiteFull, useSiteView } from '../chrome/siteView'
import { webApp, type WebAppId } from './webApps'
import './store.css'

/** An App Store app, open: its site full screen, with no address bar. */
export function WebAppScreen({ app }: { app: WebAppId }) {
  const site = webApp(app)
  const text = useText()
  const { full, exit } = useSiteFull()
  const area = useRef<HTMLDivElement>(null)
  useSiteView(app, site.url, area)
  useEffect(() => () => setSiteFull(false), [])
  return (
    <section className="app-panel web-app" aria-label={site.name}>
      <div ref={area} className={`web-app-page${full ? ' full' : ''}${exit ? ' exit' : ''}`}>
        {!isInHost() && <p>{text.chrome.outsideHost}</p>}
      </div>
      {full && (
        <button
          type="button"
          className={exit ? 'web-app-exit shown' : 'web-app-exit'}
          aria-label={text.tv.exitFullScreen}
          onClick={() => setSiteFull(false)}
        >
          ×
        </button>
      )}
    </section>
  )
}
