import { type CSSProperties, type ReactNode, useLayoutEffect } from 'react'
import { isInHost } from '../../shared/host'
import { Player } from '../apps/youtube/Player'
import { type Hole, useYouTube } from '../apps/youtube/youtube'
import { useYouTubeMusic } from '../apps/youtubeMusic/youtubeMusic'
import { Keyboard } from '../keyboard/Keyboard'
import { useCabin } from '../state/store'
import { textScale } from '../state/textSize'
import { TvBar } from './TvBar'
import { VolumeHud } from './VolumeHud'
import './transitions.css'

/** The frame every screen lives in: the stage (the screen, its bar, the player, the keyboard), `overlay` above it. */
export function Shell({ children, overlay }: { children: ReactNode; overlay?: ReactNode }) {
  // The apps' text size (state/textSize.ts), on <html> so menus portalled out of the apps take it too.
  const textSize = useCabin((state) => state.textSize)
  useLayoutEffect(() => {
    document.documentElement.style.setProperty('--text-scale', String(textScale(textSize)))
  }, [textSize])
  // A video full screen: the rest steps aside for it (apps/youtube/youtube.css). A video in small (YouTube's Now
  // playing): the stage is cut open where it shows.
  const watching = useYouTube((state) => state.open)
  const youtubeHole = useYouTube((state) => state.hole)
  const musicHole = useYouTubeMusic((state) => state.hole)
  const hole = youtubeHole ?? musicHole

  return (
    <main className="shell">
      <div className={watching ? 'stage watching' : 'stage'} style={hole && !watching && isInHost() ? holeMask(hole) : {}}>
        <div className="stage-content">
          <TvBar />
          {children}
          <Player />
          <Keyboard />
          <VolumeHud />
        </div>
      </div>
      {/* What's drawn over the video where it shows through the stage (apps/youtube/VideoHole.tsx). */}
      <div id="video-over" className="video-over" />
      {overlay}
    </main>
  )
}

/** A mask showing everything but a rounded rectangle, through which what's under the page shows. */
function holeMask({ x, y, width, height, radius }: Hole): CSSProperties {
  const shape = `<svg xmlns='http://www.w3.org/2000/svg' width='${width}' height='${height}'><rect width='${width}' height='${height}' rx='${radius}'/></svg>`
  return {
    maskImage: `linear-gradient(#000 0 0), url("data:image/svg+xml,${encodeURIComponent(shape)}")`,
    maskPosition: `0 0, ${x}px ${y}px`,
    maskSize: `100% 100%, ${width}px ${height}px`,
    maskRepeat: 'no-repeat',
    maskComposite: 'exclude',
  }
}
