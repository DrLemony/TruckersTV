import { create } from 'zustand'
import type { AppId } from '../state/apps'

/** The apps on the TV, in the row's first order. Settings is the gear at the top. */
export const APPS_SHOWN: readonly AppId[] = [
  'youtube',
  'youtubeMusic',
  'spotify',
  'exxen',
  'netflix',
  'prime',
  'disney',
  'tod',
  'tvplus',
  'tv',
  'chrome',
]

const STORAGE = 'etstv.home.order'

/** The order kept: apps no longer on the TV left out, new ones at the end. */
export function loadOrder(storage: Storage | undefined = globalThis.localStorage): AppId[] {
  try {
    const kept: unknown = JSON.parse(storage?.getItem(STORAGE) ?? '[]')
    const known = Array.isArray(kept) ? APPS_SHOWN.filter((app) => kept.includes(app)) : []
    const inOrder = known.sort((a, b) => (kept as AppId[]).indexOf(a) - (kept as AppId[]).indexOf(b))
    return [...inOrder, ...APPS_SHOWN.filter((app) => !inOrder.includes(app))]
  } catch {
    return [...APPS_SHOWN]
  }
}

/** `order` with `app` taken to where `to` is. */
export function moved(order: readonly AppId[], app: AppId, to: AppId): AppId[] {
  const next = order.filter((each) => each !== app)
  next.splice(order.indexOf(to), 0, app)
  return next
}

export const useHomeOrder = create<{ order: AppId[]; move(app: AppId, to: AppId): void }>((set, get) => ({
  order: loadOrder(),
  move: (app, to) => {
    const order = moved(get().order, app, to)
    set({ order })
    try {
      localStorage.setItem(STORAGE, JSON.stringify(order))
    } catch {
      /* Storage may be unavailable: the order stays for this session. */
    }
  },
}))
