import { useEffect, useState } from 'react'
import { type HostStatus, spotifyCheck, youtubeMusicCheck } from './useHostStatus'

/**
 * Whether each service is signed in, as the companion last knew: shown at once when it opens, kept up to date
 * by every status that says (a sign-in or out here, or the service run in the cab). Only whether, nothing
 * else of the account, in the companion's own storage (gone with TruckersTV when it's uninstalled). A service
 * never known is asked once, as the companion opens: run hidden to say.
 */
export interface SignedIn {
  youtubeMusic: boolean | null
  spotify: boolean | null
}

export type Service = keyof SignedIn

const SERVICES: Service[] = ['youtubeMusic', 'spotify']

const KEY = 'westplay.companion.signedIn'

function kept(): SignedIn {
  let value: Partial<Record<Service, unknown>> | null = null
  try {
    value = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Record<Service, unknown>> | null
  } catch {
    // Nothing kept: each is asked.
  }
  const known = (service: Service) => (typeof value?.[service] === 'boolean' ? (value[service] as boolean) : null)
  return { youtubeMusic: known('youtubeMusic'), spotify: known('spotify') }
}

function keep(value: SignedIn) {
  try {
    localStorage.setItem(KEY, JSON.stringify(value))
  } catch {
    // Not kept: known again next time a service says.
  }
}

/** What's known now: the status's, else what was kept (saved again whenever a status says). */
export function useSignedIn(status: HostStatus | null): SignedIn {
  const [known] = useState(kept)
  const now: SignedIn = {
    youtubeMusic: status?.youtubeMusicSignedIn ?? null,
    spotify: status?.spotifySignedIn ?? null,
  }
  useEffect(() => {
    if (SERVICES.every((service) => now[service] === null)) return
    const before = kept()
    keep({
      youtubeMusic: now.youtubeMusic ?? before.youtubeMusic,
      spotify: now.spotify ?? before.spotify,
    })
    // Keyed by the answers (`now` itself is new at every render).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [now.youtubeMusic, now.spotify])
  return {
    youtubeMusic: now.youtubeMusic ?? known.youtubeMusic,
    spotify: now.spotify ?? known.spotify,
  }
}

/** As the companion opens: each service never known yet is run, hidden, to say. */
export function useAskOnce(): void {
  useEffect(() => {
    const known = kept()
    if (known.youtubeMusic === null) youtubeMusicCheck().catch(() => {})
    if (known.spotify === null) spotifyCheck().catch(() => {})
  }, [])
}
