import { type ReactNode, useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'
import { command, followArea, useYouTube } from './youtube'


/**
 * Where the video playing shows outside the full-screen player (YouTube's Now playing, the Dashboard's): its
 * picture is YouTube's own, placed under the page by the host, and the screen is cut open over this
 * element (Shell.tsx), its rounding included. `crop`: the picture fills it, cropped (otherwise all of it
 * shows, with bars). Placed again whenever it may have moved.
 * Outside the app (a plain browser) the thumbnail is all there is.
 */
export function VideoHole({ className, crop = false }: { className: string; crop?: boolean }) {
  const thumbnail = useYouTube((store) => store.current?.thumbnail ?? null)
  const frame = useRef<HTMLSpanElement>(null)
  useEffect(() => {
    const element = frame.current
    if (!element) return
    const place = () => {
      const area = document.querySelector('.stage')?.getBoundingClientRect()
      const box = element.getBoundingClientRect()
      if (!area || box.width === 0) return
      const radius = parseFloat(getComputedStyle(element).borderTopLeftRadius)
      useYouTube.setState({
        hole: { x: box.left - area.left, y: box.top - area.top, width: box.width, height: box.height, radius },
      })
      const scale = window.devicePixelRatio
      command({
        action: 'show',
        x: Math.round(box.left * scale),
        y: Math.round(box.top * scale),
        width: Math.round(box.width * scale),
        height: Math.round(box.height * scale),
        crop,
      })
    }
    const following = followArea(place, element)
    // The app's window slides in (a transform, which no size change tells of): placed again as it settles.
    const settling = setTimeout(place, 220)
    return () => {
      clearTimeout(settling)
      following()
      useYouTube.setState({ hole: null })
      // Gone (whatever shows it next places it again).
      command({ action: 'hide' })
    }
  }, [crop])
  // The thumbnail, under the picture (cut out with the page while the picture shows).
  return (
    <span ref={frame} className={className}>
      {thumbnail && <img src={thumbnail} alt="" />}
    </span>
  )
}

/**
 * Content over the video where it shows through the page (`VideoHole`): drawn in its own layer above the
 * screen (anything in the screen there is cut out with it), the same size, at the same scale.
 */
export function OverVideo({ children }: { children: ReactNode }) {
  const hole = useYouTube((store) => store.hole)
  // The layer is the shell's (Shell.tsx): there by the time a hole is (a VideoHole placed after it).
  const layer = hole ? document.getElementById('video-over') : null
  if (!layer || !hole) return null
  return createPortal(
    <div
      className="over-video"
      style={{ left: hole.x, top: hole.y, width: hole.width, height: hole.height, borderRadius: hole.radius }}
    >
      <div
        className="over-video-content"
        style={{
          width: `calc(${hole.width}px / 1)`,
          height: `calc(${hole.height}px / 1)`,
        }}
      >
        {children}
      </div>
    </div>,
    layer,
  )
}
