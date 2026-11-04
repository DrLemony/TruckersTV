import type { ReactNode } from 'react'

/**
 * The Settings app's glyphs, our own drawings: white on each row's coloured tile (Apple's look, not Apple's
 * symbols), and the list's chevrons.
 */

function TileGlyph({ children }: { children: ReactNode }) {
  return (
    <svg viewBox="0 0 24 24" width="70%" height="70%" fill="#fff" aria-hidden>
      {children}
    </svg>
  )
}

/** Appearance: a circle half dark, half light. */
export function AppearanceGlyph() {
  return (
    <TileGlyph>
      <path d="M12 3.2a8.8 8.8 0 1 0 0 17.6 8.8 8.8 0 0 0 0-17.6Zm0 2a6.8 6.8 0 0 1 0 13.6Z" fillRule="evenodd" />
    </TileGlyph>
  )
}

/** Wallpaper: a flower of six petals. */
export function SoundGlyph() {
  return (
    <TileGlyph>
      <path d="M4 9.2h3.1l4.4-3.7a.7.7 0 0 1 1.15.54v11.92a.7.7 0 0 1-1.15.54L7.1 14.8H4a1 1 0 0 1-1-1V10.2a1 1 0 0 1 1-1Z" />
      <path
        d="M15.6 9a4.2 4.2 0 0 1 0 6M18.2 6.6a7.6 7.6 0 0 1 0 10.8"
        fill="none"
        stroke="#fff"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
    </TileGlyph>
  )
}

/** The chevron after a row that opens a page. */
export function ChevronGlyph({ back = false }: { back?: boolean }) {
  return (
    <svg
      viewBox="0 0 12 20"
      width="12"
      height="20"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      style={back ? { transform: 'scaleX(-1)' } : undefined}
      aria-hidden
    >
      <path d="m3 2.5 7 7.5-7 7.5" />
    </svg>
  )
}

/** The tick on a chosen option. */
export function CheckGlyph() {
  return (
    <svg
      viewBox="0 0 24 24"
      width="20"
      height="20"
      fill="none"
      stroke="currentColor"
      strokeWidth="3"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <path d="m5 12.5 4.6 4.6L19 7.5" />
    </svg>
  )
}
