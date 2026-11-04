import type { AppId } from './apps'

/** Where the TV is. */
export type Route = { kind: 'home' } | { kind: 'app'; app: AppId }

export interface NavigationState {
  route: Route
  /** Most recently used apps, newest first. */
  recents: AppId[]
}

export type NavigationAction = { type: 'openApp'; app: AppId } | { type: 'pressHome' }

export const RECENTS_LIMIT = 5

export const INITIAL_NAVIGATION: NavigationState = {
  route: { kind: 'home' },
  recents: [],
}

/** Opening an app moves it to the front of the recents. */
export function navigate(state: NavigationState, action: NavigationAction): NavigationState {
  switch (action.type) {
    case 'openApp':
      return {
        route: { kind: 'app', app: action.app },
        recents: [action.app, ...state.recents.filter((app) => app !== action.app)].slice(0, RECENTS_LIMIT),
      }
    case 'pressHome':
      return { ...state, route: { kind: 'home' } }
  }
}

/** A stable string for a route, e.g. for React keys. */
export function routeKey(route: Route): string {
  return route.kind === 'app' ? `app:${route.app}` : route.kind
}
