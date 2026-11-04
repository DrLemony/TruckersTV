import { create } from 'zustand'

/**
 * The App Store's apps: streaming services' and sites' own web pages, each an app of its own (state/apps.ts)
 * that shows its site full screen in the site WebView (apps/chrome/siteView.ts: it draws protected video where
 * the screen's capture sees it). All of them are on the TV's Home.
 */
export const WEB_APP_IDS = ['youtube', 'exxen', 'netflix', 'disney', 'tod', 'prime', 'tvplus'] as const

export type WebAppId = (typeof WEB_APP_IDS)[number]

export interface WebApp {
  id: WebAppId
  name: string
  url: string
}

const SITES: Record<WebAppId, Omit<WebApp, 'id'>> = {
  youtube: { name: 'YouTube', url: 'https://www.youtube.com/' },
  exxen: { name: 'Exxen', url: 'https://www.exxen.com/' },
  netflix: { name: 'Netflix', url: 'https://www.netflix.com/' },
  disney: { name: 'Disney+', url: 'https://www.disneyplus.com/' },
  tod: { name: 'TOD', url: 'https://www.todtv.com.tr/' },
  prime: { name: 'Prime Video', url: 'https://www.primevideo.com/' },
  tvplus: { name: 'TV+', url: 'https://tvplus.com.tr/' },
}

export function webApp(id: WebAppId): WebApp {
  return { id, ...SITES[id] }
}

export const WEB_APPS: readonly WebApp[] = WEB_APP_IDS.map(webApp)

/** Their names, the same in every language (state/strings.ts). */
export const WEB_APP_NAMES = Object.fromEntries(WEB_APPS.map((app) => [app.id, app.name])) as Record<WebAppId, string>

export function isWebAppId(value: unknown): value is WebAppId {
  return typeof value === 'string' && (WEB_APP_IDS as readonly string[]).includes(value)
}

/** A site's own icon, fetched as the page shows it: no service's mark is kept in TruckersTV. */
export function webAppIcon(app: WebApp): string {
  return `https://www.google.com/s2/favicons?sz=128&domain_url=${encodeURIComponent(app.url)}`
}

const STORAGE = 'westplay.store.apps'

/** The apps the player got, in the order they got them (ids the catalog no longer has left out). */
export function loadInstalled(storage: Storage | undefined = globalThis.localStorage): WebAppId[] {
  try {
    const kept: unknown = JSON.parse(storage?.getItem(STORAGE) ?? '[]')
    return Array.isArray(kept) ? [...new Set(kept.filter(isWebAppId))] : []
  } catch {
    return []
  }
}

function save(installed: WebAppId[]) {
  try {
    localStorage.setItem(STORAGE, JSON.stringify(installed))
  } catch {
    /* Storage may be unavailable: the apps stay for this session. */
  }
}

interface Installed {
  installed: WebAppId[]
  get(id: WebAppId): void
  remove(id: WebAppId): void
}

export const useWebApps = create<Installed>((set, get) => ({
  installed: loadInstalled(),
  get: (id) => {
    if (get().installed.includes(id)) return
    const installed = [...get().installed, id]
    save(installed)
    set({ installed })
  },
  remove: (id) => {
    const installed = get().installed.filter((kept) => kept !== id)
    save(installed)
    set({ installed })
  },
}))
