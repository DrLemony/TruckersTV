import type { CSSProperties } from 'react'
import { flushSync } from 'react-dom'
import { create } from 'zustand'
import type { AppId } from './apps'
import type { Route } from './navigation'

/**
 * App launch and close animations: an app grows out of the icon that opened it and shrinks back
 * into its Home icon when it closes. Built on the View Transitions API: the browser snapshots the screen
 * before and after a navigation and animates between them; elements that carry the same transition name in
 * both snapshots morph into each other (the icon into the app window, or back). The motion is in
 * shell/transitions.css.
 */

/** Where the player clicked to open an app. */
export type LaunchOrigin = 'home'

/** An app window animating between an icon and its full size. */
export interface Launch {
  app: AppId
  /** The icon it grows out of (`open`) or shrinks into (`close`). */
  origin: LaunchOrigin
  direction: 'open' | 'close'
}

/** What a navigation animates; `null` for a plain crossfade. */
export function launchFor(before: Route, after: Route, origin: LaunchOrigin | null): Launch | null {
  if (after.kind === 'app' && origin !== null && !(before.kind === 'app' && before.app === after.app)) {
    return { app: after.app, origin, direction: 'open' }
  }
  // Closing, the app shrinks into its Home icon.
  if (before.kind === 'app' && after.kind !== 'app') return { app: before.app, origin: 'home', direction: 'close' }
  return null
}

/**
 * The kind of screen change, for its motion (`html[data-transition]` in shell/transitions.css): an app
 * opening or closing, or a crossfade.
 */
export type TransitionKind = 'open' | 'close' | 'switch'

export function transitionKind(launch: Launch | null): TransitionKind {
  return launch?.direction ?? 'switch'
}

/** The transition name shared by the morphing icon and app window. */
const APP_WINDOW = 'app-window'

/** An element that can take part in a launch: an app's icon on Home, or its window. */
export type LaunchRole = { kind: 'icon'; origin: LaunchOrigin } | { kind: 'window' }

interface LaunchState {
  launch: Launch | null
  /** Which snapshot is being prepared: before the navigation (`from`) or after it (`to`). */
  phase: 'from' | 'to'
}

const useLaunch = create<LaunchState>()(() => ({ launch: null, phase: 'from' }))

/**
 * Whether this element is the morphing one in the snapshot being prepared. Opening, the icon is it before
 * the navigation and the window after; closing, the other way round. Only one element may carry a name in
 * each snapshot.
 */
export function carriesLaunch(state: LaunchState, app: AppId, role: LaunchRole): boolean {
  const { launch, phase } = state
  if (!launch || launch.app !== app) return false
  const iconPhase = launch.direction === 'open' ? 'from' : 'to'
  if (role.kind === 'icon') return role.origin === launch.origin && phase === iconPhase
  return phase !== iconPhase
}

/** Whether this element is the morphing one in the snapshot being prepared (see `carriesLaunch`). */
export function useCarriesLaunch(app: AppId, role: LaunchRole): boolean {
  return useLaunch((state) => carriesLaunch(state, app, role))
}

/** Style for an icon or app window: the shared transition name while it takes part in a launch. */
export function useLaunchStyle(app: AppId, role: LaunchRole): CSSProperties | undefined {
  return useCarriesLaunch(app, role) ? LAUNCH_STYLE : undefined
}

/** What the morphing element wears. */
export const LAUNCH_STYLE: CSSProperties = { viewTransitionName: APP_WINDOW }

let running: ViewTransition | null = null

/**
 * Calls `next` once no screen change is animating any more: a running one is finished at once (it jumps to
 * its end), and `next` runs after its navigation has been applied, so it sees the current screen. Several
 * calls run in order.
 */
export function afterRunningTransition(next: () => void): void {
  if (!running) {
    next()
    return
  }
  running.skipTransition()
  running.updateCallbackDone.catch(() => {}).finally(next)
}

/**
 * Runs `update` (a navigation) as an animated transition. `update` must change the screen
 * synchronously. Without View Transitions support it just runs. Start it from `afterRunningTransition`, so
 * it begins from the current screen.
 */
export function animateNavigation(launch: Launch | null, update: () => void): void {
  if (typeof document.startViewTransition !== 'function') {
    update()
    return
  }
  running?.skipTransition()

  const root = document.documentElement
  flushSync(() => useLaunch.setState({ launch, phase: 'from' }))
  root.dataset.transition = transitionKind(launch)

  const transition = document.startViewTransition(() =>
    flushSync(() => {
      useLaunch.setState({ phase: 'to' })
      update()
    }),
  )
  running = transition
  transition.finished.finally(() => {
    if (running !== transition) return
    running = null
    delete root.dataset.transition
    useLaunch.setState({ launch: null, phase: 'from' })
  })
}
