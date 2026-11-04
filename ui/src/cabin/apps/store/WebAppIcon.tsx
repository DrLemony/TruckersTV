import { type CSSProperties, useState } from 'react'
import { type WebApp, webAppIcon } from './webApps'

/** A site's own icon, or its first letter while it can't be fetched (no internet). */
export function WebAppIcon({ app, className, style }: { app: WebApp; className?: string; style?: CSSProperties }) {
  const [failed, setFailed] = useState(false)
  return failed ? (
    <span className={`${className} letter`} style={style} aria-hidden>
      {app.name[0]}
    </span>
  ) : (
    <img className={className} style={style} src={webAppIcon(app)} alt="" onError={() => setFailed(true)} />
  )
}
