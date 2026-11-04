import { useEffect } from 'react'
import { create } from 'zustand'
import { type BrowserCommand, isInHost, postToHost, subscribeHostMessages } from '../../../shared/host'
import { useKeyboard } from '../../keyboard/store'
import { useLanguage } from '../../state/text'
import { nativeRect } from '../nativeRect'

/**
 * The site WebView (crates/host/src/site_browser.rs, one for the whole cabin), shared by Chrome and the App
 * Store's apps: the open one shows it over its own area, at its own page (each one's last, kept), all signed in
 * to the same profile. Opening another moves the view to that one's page.
 */
export const browserCommand = (command: BrowserCommand) => postToHost({ type: 'browser', command })

/** The site apps' sign-in: a window open on the desktop, and the apps signed in. */
export const useSiteSignIn = create<{ open: boolean; signedIn: string[] }>(() => ({ open: false, signedIn: [] }))

/** A site app on the whole TV, and the X out of it showing. */
export const useSiteFull = create<{ full: boolean; exit: boolean }>(() => ({ full: false, exit: false }))

/** How long the full screen's X stays, ms. */
const EXIT_SHOWN_MS = 3000
let exitHides: ReturnType<typeof setTimeout> | undefined

/** Back from the full screen, or (`full`) into it, the X gone. */
export function setSiteFull(full: boolean) {
  clearTimeout(exitHides)
  useSiteFull.setState({ full, exit: false })
}

// Whose page the view shows (`null`: none yet, or closed).
let owner: string | null = null
const REMOTE_ID = 'browser-page'

function storageKey(who: string) {
  return who === 'chrome' ? 'westplay.browser.url' : `westplay.web.${who}.url`
}

/** The page `who` was last at, or its `home`. */
export function savedPage(who: string, home: string): string {
  try {
    return localStorage.getItem(storageKey(who)) || home
  } catch {
    return home
  }
}

/** The view closed (Chrome's restart): whoever shows it next opens its page again. */
export function forgetSiteView() {
  owner = null
}

/** Shows the site view over `area` while mounted, at `who`'s page; a new `generation` places it afresh. */
export function useSiteView(who: string, home: string, area: { current: HTMLElement | null }, generation = 0) {
  const language = useLanguage()

  useEffect(
    () =>
      subscribeHostMessages((message) => {
        if (message.type === 'browserSignIn') useSiteSignIn.setState({ open: message.open, signedIn: message.signedIn })
        if (owner !== who) return
        if (message.type === 'browserPress' && useSiteFull.getState().full) {
          clearTimeout(exitHides)
          useSiteFull.setState({ exit: true })
          exitHides = setTimeout(() => useSiteFull.setState({ exit: false }), EXIT_SHOWN_MS)
        }
        if (message.type === 'browserState' && message.url) {
          try {
            localStorage.setItem(storageKey(who), message.url)
          } catch {
            /* Storage may be unavailable. */
          }
        }
        if (message.type === 'browserFocus') {
          if (message.editable) {
            useKeyboard.getState().attach(
              {
                id: REMOTE_ID,
                returnKind: 'done',
                read: () => '',
                write: () => {},
                submit: () => browserCommand({ action: 'input', event: { key: 'enter' } }),
                send: (event) => browserCommand({ action: 'input', event }),
              },
              language,
            )
          } else useKeyboard.getState().detach(REMOTE_ID)
        }
      }),
    [who, language],
  )

  useEffect(() => {
    if (!isInHost()) return
    let gone = false
    let frame = 0
    let last = ''
    if (owner !== who) {
      owner = who
      browserCommand({ action: 'navigate', url: savedPage(who, home) })
    }
    const place = () => {
      if (gone) return
      const rect = nativeRect(area.current)
      if (rect) {
        const next = JSON.stringify(rect)
        if (last !== next) {
          last = next
          browserCommand(rect.width && rect.height ? { action: 'show', ...rect } : { action: 'hide' })
        }
      }
      frame = requestAnimationFrame(place)
    }
    place()
    return () => {
      gone = true
      cancelAnimationFrame(frame)
      browserCommand({ action: 'hide' })
      useKeyboard.getState().detach(REMOTE_ID)
    }
  }, [who, home, area, generation])
}

/** Takes the keyboard off the page (Chrome's own field is about to have it). */
export function detachSiteKeyboard() {
  useKeyboard.getState().detach(REMOTE_ID)
}
