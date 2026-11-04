import type { AppIconRole } from '../../shared/icons'
import { WEB_APP_IDS } from '../apps/store/webApps'

/** Every app on the TV. */
export const APP_IDS = [
  'spotify',
  'youtubeMusic',
  'chrome',
  'tv',
  'settings',
  // The sites' own pages (apps/store/webApps.ts).
  ...WEB_APP_IDS,
] as const

export type AppId = (typeof APP_IDS)[number]

/** An app's icon (`null`: its site's own; its name is in the TV's language: state/text.ts). */
export const APP_ICONS: Record<AppId, AppIconRole | null> = {
  spotify: 'spotify',
  youtubeMusic: 'youtubeMusic',
  chrome: 'chrome',
  tv: 'tv',
  settings: 'settings',
  ...(Object.fromEntries(WEB_APP_IDS.map((id) => [id, null])) as Record<(typeof WEB_APP_IDS)[number], null>),
  youtube: 'youtube',
}

export function isAppId(value: unknown): value is AppId {
  return typeof value === 'string' && (APP_IDS as readonly string[]).includes(value)
}
