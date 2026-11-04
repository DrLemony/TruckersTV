import { call, on } from './bridge'
import { useEffect, useState } from 'react'

/** Emitted by the Tauri backend about once per second (mirrors `westplay_host::Status`). */
export interface HostStatus {
  webviewVersion: string
  adapter: string | null
  /** The graphics card TruckersTV's screen is drawn on, and the game's (`null` until the game runs). */
  cabAdapter: string | null
  gameAdapter: string | null
  framesPerSecond: number
  droppedTotal: number
  gameConnected: boolean
  screenDrawing: boolean
  /** Whether the truck the player is in is one TruckersTV shows in, as the plugin last saw (`null`: not known yet). */
  truckSupported: boolean | null
  /** Whether YouTube is signed in (`null`: not known yet: YouTube hasn't run), and whether Google's sign-in
   * window is open. Nothing else of the account. */
  youtubeSignedIn: boolean | null
  youtubeSigningIn: boolean
  /** The site app whose sign-in window is open, by its id, and those signed in. */
  siteSigningIn: string | null
  siteSignedIn: string[]
  /** The same of Spotify: signed in or not (`null`: not known yet), and its sign-in window open. */
  spotifySignedIn: boolean | null
  spotifySigningIn: boolean
  error: string | null
}

/** Latest host status, or `null` until the first update arrives. */
export function useHostStatus(): HostStatus | null {
  const [status, setStatus] = useState<HostStatus | null>(null)
  useEffect(() => {
    return on<HostStatus>('host-status', setStatus)
  }, [])
  return status
}

/** Opens a site app's sign-in in a window on the desktop. */
export function siteSignIn(app: string): Promise<void> {
  return call('site_sign_in', { app })
}

/** Opens Google's sign-in for YouTube in a window on the desktop. */
export function youtubeSignIn(): Promise<void> {
  return call('youtube_sign_in')
}

/** Signs YouTube out. */
export function youtubeSignOut(): Promise<void> {
  return call('youtube_sign_out')
}

/** Runs YouTube (hidden), so the status says whether it's signed in. */
export function youtubeCheck(): Promise<void> {
  return call('youtube_check')
}

/** Opens Spotify's sign-in in a window on the desktop. */
export function spotifySignIn(): Promise<void> {
  return call('spotify_sign_in')
}

/** Signs Spotify out. */
export function spotifySignOut(): Promise<void> {
  return call('spotify_sign_out')
}

/** Runs Spotify's web player (hidden), so the status says whether it's signed in. */
export function spotifyCheck(): Promise<void> {
  return call('spotify_check')
}
