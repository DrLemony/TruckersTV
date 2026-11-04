import { useEffect, useRef, useState } from 'react'
import { isInHost, subscribeHostMessages, type HostMessage } from '../../../shared/host'
import { TextField } from '../../keyboard/TextField'
import { useKeyboard } from '../../keyboard/store'
import { useText } from '../../state/text'
import { browserCommand as command, detachSiteKeyboard, forgetSiteView, savedPage, useSiteView } from './siteView'
import './chrome.css'

type BrowserState = Extract<HostMessage, { type: 'browserState' }>
const INITIAL: BrowserState = {
  type: 'browserState',
  url: '',
  title: '',
  loading: false,
  back: false,
  forward: false,
  error: null,
}
const HOME = 'https://www.google.com/'
let current = INITIAL
let currentZoom = 1

/** The browser surface is the site view (./siteView.ts). This page owns only its controls. */
export function ChromeApp() {
  const text = useText().chrome
  const [state, setState] = useState(current)
  const [address, setAddress] = useState(() => savedPage('chrome', HOME))
  const [zoom, setZoom] = useState(currentZoom)
  const [generation, setGeneration] = useState(0)
  const area = useRef<HTMLDivElement>(null)
  const typing = useKeyboard((state) => state.field !== null)
  useSiteView('chrome', HOME, area, generation)

  useEffect(
    () =>
      subscribeHostMessages((message) => {
        if (message.type !== 'browserState') return
        current = message
        setState(message)
        if (message.url && useKeyboard.getState().field?.id !== 'browser-address') setAddress(message.url)
      }),
    [],
  )

  const navigate = () => {
    useKeyboard.getState().detach('browser-address')
    detachSiteKeyboard()
    command({ action: 'navigate', url: address })
  }
  const changeZoom = (delta: number) => {
    const factor = Math.max(0.5, Math.min(2, Math.round((zoom + delta) * 10) / 10))
    setZoom(factor)
    currentZoom = factor
    command({ action: 'zoom', factor })
  }
  return (
    <section className={typing ? 'app-panel chrome-app typing' : 'app-panel chrome-app'} aria-label={useText().apps.chrome}>
      <div ref={area} className="chrome-page">
        {!isInHost() && <p>{text.outsideHost}</p>}
      </div>
      {state.error && (
        <div className="chrome-error" role="alert">
          {state.error}
        </div>
      )}
      <div className="chrome-bar">
        <button aria-label={text.back} disabled={!state.back} onClick={() => command({ action: 'back' })}>
          ‹
        </button>
        <button aria-label={text.forward} disabled={!state.forward} onClick={() => command({ action: 'forward' })}>
          ›
        </button>
        <div className={state.loading ? 'chrome-address loading' : 'chrome-address'}>
          <TextField
            id="browser-address"
            value={address}
            onChange={setAddress}
            onSubmit={navigate}
            returnKind="go"
            hint={text.address}
            label={text.address}
          />
          <button
            aria-label={state.loading ? text.stop : text.reload}
            onClick={() => command({ action: state.loading ? 'stop' : 'reload' })}
          >
            {state.loading ? '×' : '↻'}
          </button>
        </div>
        <button aria-label={text.zoomOut} disabled={zoom <= 0.5} onClick={() => changeZoom(-0.1)}>
          a
        </button>
        <span className="chrome-zoom">{Math.round(zoom * 100)}%</span>
        <button aria-label={text.zoomIn} disabled={zoom >= 2} onClick={() => changeZoom(0.1)}>
          A
        </button>
        <button
          aria-label={text.restart}
          onClick={() => {
            command({ action: 'close' })
            forgetSiteView()
            currentZoom = 1
            setZoom(1)
            setGeneration((value) => value + 1)
          }}
        >
          ⟲
        </button>
      </div>
    </section>
  )
}
