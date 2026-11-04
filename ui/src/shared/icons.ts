/**
 * App icons by role: the music and video services' own marks, from their official files, and the app's own.
 */
export type AppIconRole = 'spotify' | 'settings' | 'youtube' | 'youtubeMusic' | 'chrome' | 'tv'

/** Each role's file's name. */
const FILES: Record<AppIconRole, string> = {
  chrome: 'chrome',
  tv: 'tv',
  spotify: 'spotify',
  settings: 'settings',
  youtube: 'youtube',
  youtubeMusic: 'youtube-music',
}

export function appIcon(role: AppIconRole): string {
  return `/icons/${FILES[role]}.svg`
}
