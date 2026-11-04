import type { PowerPhase } from '../state/power'
import './boot.css'

/** Before the TV is on: its mark and name as it starts, black while it's off. */
export function BootScreen({ phase }: { phase: Exclude<PowerPhase, 'on'> }) {
  return (
    <div className="boot" aria-hidden>
      {phase === 'starting' && (
        <div className="boot-mark">
          <img src="/logo.svg" alt="" />
          <span>
            Truckers<b>TV</b>
          </span>
        </div>
      )}
    </div>
  )
}
