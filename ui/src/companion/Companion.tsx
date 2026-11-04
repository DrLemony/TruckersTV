import { type ReactNode, useCallback, useEffect, useLayoutEffect, useState } from 'react'
import { WEB_APPS } from '../cabin/apps/store/webApps'
import { type Service, type SignedIn, useAskOnce, useSignedIn } from './accounts'
import { appVersion, call } from './bridge'
import '@fontsource-variable/overpass'
import './companion.css'
import { KeyButton } from './KeyButton'
import {
  checkGame,
  chooseGameFolder,
  finishSetup,
  type GameCheck,
  type GameInfo,
  gameRunning,
  type InstallReport,
  installIntoGame,
  leavePc,
  removeData,
  type SetupState,
  setupState,
  uninstallFromGame,
} from './setup'
import {
  type HostStatus,
  siteSignIn,
  spotifySignIn,
  spotifySignOut,
  useHostStatus,
  youtubeMusicSignIn,
  youtubeMusicSignOut,
} from './useHostStatus'
import { type Binding, DEFAULT_KEY, KEYBIND_IDS, type KeybindId, SENSITIVITY, useSettings } from './useSettings'
import { type Words, wordsFor } from './words'

const PAGES = ['status', 'keys', 'accounts', 'install'] as const
type Page = (typeof PAGES)[number]

/** The window: its first-run setup, then the pages down its left side. */
export function Companion() {
  const [setup, setSetup] = useState<SetupState | null>(null)
  const [error, setError] = useState<string | null>(null)
  const status = useHostStatus()
  const refresh = useCallback(() => {
    setupState()
      .then(setSetup)
      .catch((e: unknown) => setError(String(e)))
  }, [])
  useEffect(refresh, [refresh])
  useShowWhenReady(setup !== null || error !== null)
  if (!setup) return <main className="tt tt-blank">{error}</main>
  const w = wordsFor(setup.language)
  return setup.finished ? (
    <Main setup={setup} status={status} w={w} refresh={refresh} />
  ) : (
    <FirstRun setup={setup} w={w} done={refresh} />
  )
}

/** The window shows once its first page is drawn. */
function useShowWhenReady(ready: boolean) {
  useLayoutEffect(() => {
    if (ready) requestAnimationFrame(() => void call('companion_ready').catch(() => {}))
  }, [ready])
}

function Main({
  setup,
  status,
  w,
  refresh,
}: {
  setup: SetupState
  status: HostStatus | null
  w: Words
  refresh(): void
}) {
  const [page, setPage] = useState<Page>('status')
  useFinishUpdate(setup, refresh)
  return (
    <main className="tt">
      <nav className="tt-rail">
        <div className="tt-brand">
          <img src="/logo.svg" alt="" />
          <span>
            Truckers<b>TV</b>
          </span>
        </div>
        <ol>
          {PAGES.map((each, index) => (
            <li key={each}>
              <button type="button" className={each === page ? 'on' : undefined} onClick={() => setPage(each)}>
                <i>{String(index + 1).padStart(2, '0')}</i>
                {w.nav[each]}
              </button>
            </li>
          ))}
        </ol>
        <i className="tt-road" />
      </nav>
      <section className="tt-page" key={page}>
        {page === 'status' && <StatusPage status={status} w={w} />}
        {page === 'keys' && <KeysPage w={w} />}
        {page === 'accounts' && <AccountsPage status={status} w={w} />}
        {page === 'install' && <InstallPage setup={setup} w={w} refresh={refresh} />}
      </section>
    </main>
  )
}

/** A new version's plugin and mod go into the game on their own, once it's closed. */
function useFinishUpdate(setup: SetupState, refresh: () => void) {
  const running = useGameRunning()
  const dir = setup.updated ? setup.game?.dir : undefined
  useEffect(() => {
    if (!dir || running !== false) return
    installIntoGame(dir, false).then((report) => report.done && refresh(), () => {})
  }, [dir, running, refresh])
}

/** Whether the game is running, looked at every couple of seconds (`null`: not known yet). */
function useGameRunning(): boolean | null {
  const [running, setRunning] = useState<boolean | null>(null)
  useEffect(() => {
    const look = () => void gameRunning().then(setRunning, () => {})
    look()
    const timer = window.setInterval(look, 2000)
    return () => window.clearInterval(timer)
  }, [])
  return running
}

function Row({ title, detail, children }: { title: string; detail?: string | null; children?: ReactNode }) {
  return (
    <div className="tt-row">
      <span>
        <b>{title}</b>
        {detail && <small>{detail}</small>}
      </span>
      {children}
    </div>
  )
}

function Heading({ title, sub }: { title: string; sub: string }) {
  return (
    <header className="tt-heading">
      <h1>{title}</h1>
      <p>{sub}</p>
    </header>
  )
}

type ScreenState = keyof Words['state']

function screenState(status: HostStatus | null): ScreenState {
  if (!status) return 'starting'
  if (status.error) return 'stopped'
  if (!status.gameConnected) return 'noGame'
  if (status.truckSupported === false) return 'unsupported'
  return status.screenDrawing ? 'live' : 'notInCab'
}

function StatusPage({ status, w }: { status: HostStatus | null; w: Words }) {
  const state = screenState(status)
  const { settings, update } = useSettings()
  const videos = settings?.tv.videos ?? null
  const setVideos = (folder: string | null) => {
    if (settings) void update({ ...settings, tv: { ...settings.tv, videos: folder } })
  }
  return (
    <>
      <div className={`tt-state ${state}`}>
        <i />
        <div>
          <h1>{w.state[state][0]}</h1>
          <p>{w.state[state][1]}</p>
        </div>
        {state === 'live' && <span>{w.framesValue(status?.framesPerSecond ?? 0)}</span>}
      </div>
      {status?.error && <p className="tt-note bad">{status.error}</p>}
      <h2>{w.howTo}</h2>
      <ol className="tt-steps">
        {w.steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ol>
      <Row title={w.videos} detail={videos ?? w.videosDetail}>
        {videos && (
          <button type="button" className="tt-button quiet" onClick={() => setVideos(null)}>
            {w.clear}
          </button>
        )}
        <button
          type="button"
          className="tt-button quiet"
          onClick={() => void call<string | null>('pick_tv_videos', { title: w.videos }).then((f) => f && setVideos(f))}
        >
          {w.choose}
        </button>
      </Row>
    </>
  )
}

function KeysPage({ w }: { w: Words }) {
  const { settings, update, error } = useSettings()
  const k = w.keys
  const [listening, setListening] = useState<KeybindId | 'screen' | null>(null)
  const [refused, setRefused] = useState<string | null>(null)
  const listen = useCallback(
    (which: KeybindId | 'screen') => (on: boolean) => {
      setListening(on ? which : null)
      if (on) setRefused(null)
    },
    [],
  )
  if (!settings) return <Heading title={k.title} sub={k.sub} />
  const screenKey = settings.interaction.key ?? DEFAULT_KEY
  const names: Record<KeybindId, string> = {
    volumeUp: k.volumeUp,
    volumeDown: k.volumeDown,
    playPause: k.playPause,
    zoom: k.zoom,
  }
  // What `binding` would clash with, if anything: the TV's key, or another keybind pressed the same way.
  const clash = (binding: Binding, except: KeybindId | 'screen'): string | null => {
    if (except !== 'screen' && binding.key === screenKey) return k.useScreen
    for (const id of KEYBIND_IDS) {
      if (id === except) continue
      const other = settings.keybinds[id]
      const same = other.key === binding.key && other.withInteractionKey === binding.withInteractionKey
      if (except === 'screen' ? other.key === binding.key : same) return names[id]
    }
    return null
  }
  const choose = (which: KeybindId | 'screen') => (binding: Binding) => {
    const taken = clash(binding, which)
    if (taken) return setRefused(k.inUse(taken))
    setRefused(null)
    if (which === 'screen') void update({ ...settings, interaction: { ...settings.interaction, key: binding.key } })
    else void update({ ...settings, keybinds: { ...settings.keybinds, [which]: binding } })
  }
  return (
    <>
      <Heading title={k.title} sub={k.sub} />
      <Row title={k.useScreen}>
        <KeyButton
          binding={{ key: screenKey, withInteractionKey: false }}
          label={k.useScreen}
          listening={listening === 'screen'}
          listen={listen('screen')}
          choose={choose('screen')}
          w={w}
        />
      </Row>
      <Row title={k.toggle} detail={k.toggleDetail}>
        <Switch
          on={settings.interaction.mode === 'toggle'}
          label={k.toggle}
          change={(on) =>
            void update({ ...settings, interaction: { ...settings.interaction, mode: on ? 'toggle' : 'hold' } })
          }
        />
      </Row>
      <SpeedRow
        saved={settings.interaction.sensitivity}
        label={k.speed}
        save={(sensitivity) => void update({ ...settings, interaction: { ...settings.interaction, sensitivity } })}
      />
      {KEYBIND_IDS.map((id) => (
        <Row key={id} title={names[id]}>
          <KeyButton
            binding={settings.keybinds[id]}
            with={screenKey}
            label={names[id]}
            listening={listening === id}
            listen={listen(id)}
            choose={choose(id)}
            w={w}
          />
        </Row>
      ))}
      {(refused ?? error) && <p className="tt-note bad">{refused ?? error}</p>}
    </>
  )
}

function Switch({ on, label, change }: { on: boolean; label: string; change(on: boolean): void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className={on ? 'tt-switch on' : 'tt-switch'}
      onClick={() => change(!on)}
    >
      <i />
    </button>
  )
}

/** The pointer's speed: its slider follows the thumb, and it's saved once it's let go. */
function SpeedRow({ saved, label, save }: { saved: number; label: string; save(value: number): void }) {
  const [dragging, setDragging] = useState<number | null>(null)
  const value = dragging ?? saved
  const commit = () => {
    if (dragging !== null) save(dragging)
    setDragging(null)
  }
  return (
    <Row title={label}>
      <span className="tt-value">{`${value.toFixed(1)}×`}</span>
      <input
        type="range"
        className="tt-range"
        aria-label={label}
        min={SENSITIVITY.min}
        max={SENSITIVITY.max}
        step={SENSITIVITY.step}
        value={value}
        onChange={(event) => setDragging(Number(event.target.value))}
        onPointerUp={commit}
        onKeyUp={commit}
      />
    </Row>
  )
}

const SERVICES: {
  id: Service
  name: string
  signIn(): Promise<void>
  signOut(): Promise<void>
  signingIn(status: HostStatus | null): boolean
}[] = [
  {
    id: 'youtubeMusic',
    name: 'YouTube Music',
    signIn: youtubeMusicSignIn,
    signOut: youtubeMusicSignOut,
    signingIn: (status) => status?.youtubeMusicSigningIn ?? false,
  },
  {
    id: 'spotify',
    name: 'Spotify',
    signIn: spotifySignIn,
    signOut: spotifySignOut,
    signingIn: (status) => status?.spotifySigningIn ?? false,
  },
]

function AccountsPage({ status, w }: { status: HostStatus | null; w: Words }) {
  const a = w.accounts
  const signedIn: SignedIn = useSignedIn(status)
  useAskOnce()
  return (
    <>
      <Heading title={a.title} sub={a.sub} />
      {SERVICES.map((service) => {
        const on = signedIn[service.id] === true
        const opening = service.signingIn(status)
        return (
          <Row
            key={service.id}
            title={service.name}
            detail={opening ? a.finishInWindow : on ? a.signedIn : a.notSignedIn}
          >
            <button
              type="button"
              className={on ? 'tt-button quiet' : 'tt-button'}
              disabled={opening}
              onClick={() => void (on ? service.signOut() : service.signIn())}
            >
              {on ? a.signOut : a.signIn}
            </button>
          </Row>
        )
      })}
      {WEB_APPS.map((app) => {
        const opening = status?.siteSigningIn === app.id
        const on = status?.siteSignedIn.includes(app.id) ?? false
        return (
          <Row key={app.id} title={app.name} detail={opening ? a.finishInWindow : on ? a.signedIn : a.notSignedIn}>
            <button
              type="button"
              className={on ? 'tt-button quiet' : 'tt-button'}
              disabled={opening}
              onClick={() => void siteSignIn(app.id)}
            >
              {on ? a.signInAgain : a.signIn}
            </button>
          </Row>
        )
      })}
    </>
  )
}

/** What the install said, for the player. */
function installNote(report: InstallReport, w: Words): string | null {
  const i = w.install
  if (report.modError) return report.modError
  switch (report.plugin.state) {
    case 'installed':
      return null
    case 'inUse':
      return i.inUse
    case 'needsAdministrator':
      return i.needsAdministrator
    case 'declined':
      return i.declined
    case 'failed':
      return report.plugin.error
  }
}

/** Installing into the game: what it said, whether it needs Windows' permission next, and doing it. */
function useInstall(dir: string | undefined, installed: () => void) {
  const [working, setWorking] = useState(false)
  const [report, setReport] = useState<InstallReport | null>(null)
  const [error, setError] = useState<string | null>(null)
  const state = report?.plugin.state
  const administrator = state === 'needsAdministrator' || state === 'declined'
  const install = async () => {
    if (!dir) return
    setWorking(true)
    setError(null)
    try {
      const result = await installIntoGame(dir, administrator)
      setReport(result)
      if (result.done) installed()
    } catch (e) {
      setError(String(e))
    } finally {
      setWorking(false)
    }
  }
  return { working, report, error, administrator, install }
}

function InstallPage({ setup, w, refresh }: { setup: SetupState; w: Words; refresh(): void }) {
  const i = w.install
  const game = setup.game
  const running = useGameRunning()
  const [check, setCheck] = useState<GameCheck | null>(null)
  const [version, setVersion] = useState('')
  const [leaving, setLeaving] = useState(false)
  const look = useCallback(() => {
    if (game) checkGame(game.dir).then(setCheck, () => {})
  }, [game])
  useEffect(look, [look])
  useEffect(() => void appVersion().then(setVersion), [])
  const installing = useInstall(game?.dir, () => {
    look()
    refresh()
  })
  const note = installing.report && installNote(installing.report, w)
  return (
    <>
      <Heading title={i.title} sub={i.sub} />
      {game && <Row title={i.folder} detail={game.dir} />}
      {game && <Row title={i.version} detail={game.version} />}
      <Row title={i.plugin} detail={check ? i.pluginStates[check.plugin] : '…'}>
        <button
          type="button"
          className="tt-button"
          disabled={!game || running !== false || installing.working}
          onClick={() => void installing.install()}
        >
          {installing.administrator ? i.asAdministrator : i.reinstall}
        </button>
      </Row>
      {running && <p className="tt-note">{i.closeGame}</p>}
      {installing.report?.done && <p className="tt-note">{i.installed}</p>}
      {(note ?? installing.error) && <p className="tt-note bad">{note ?? installing.error}</p>}
      <Row title={i.app} detail={`${version} · ${i.origin}`}>
        <button type="button" className="tt-button danger" onClick={() => setLeaving(true)}>
          {i.uninstall}
        </button>
      </Row>
      {leaving && <Uninstall w={w} running={running} close={() => setLeaving(false)} />}
    </>
  )
}

function Uninstall({ w, running, close }: { w: Words; running: boolean | null; close(): void }) {
  const u = w.uninstall
  const [keep, setKeep] = useState(false)
  const [working, setWorking] = useState(false)
  const [needsAdministrator, setNeedsAdministrator] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const go = async () => {
    setWorking(true)
    setError(null)
    try {
      const report = await uninstallFromGame(needsAdministrator)
      if (report.done) {
        await removeData(keep)
        await leavePc(keep)
        return
      }
      const state = report.plugin.state
      setNeedsAdministrator(state === 'needsAdministrator' || state === 'declined')
      setError(report.modError ?? (state === 'failed' ? report.plugin.error : w.install.closeGame))
    } catch (e) {
      setError(String(e))
    }
    setWorking(false)
  }
  return (
    <div className="tt-veil" onClick={working ? undefined : close}>
      <div className="tt-sheet" role="dialog" aria-label={u.title} onClick={(event) => event.stopPropagation()}>
        <h2>{u.title}</h2>
        <p>{u.text}</p>
        <Row title={u.keep}>
          <Switch on={keep} label={u.keep} change={setKeep} />
        </Row>
        {running && <p className="tt-note">{w.install.closeGame}</p>}
        {error && <p className="tt-note bad">{error}</p>}
        <div className="tt-sheet-actions">
          <button type="button" className="tt-button quiet" disabled={working} onClick={close}>
            {u.cancel}
          </button>
          <button type="button" className="tt-button danger" disabled={working || running !== false} onClick={() => void go()}>
            {working ? u.working : needsAdministrator ? w.install.asAdministrator : u.confirm}
          </button>
        </div>
      </div>
    </div>
  )
}

/** The first run: the game found (or chosen), the plugin put in, and on to the pages. */
function FirstRun({ setup, w, done }: { setup: SetupState; w: Words; done(): void }) {
  const s = w.setup
  const [game, setGame] = useState<GameInfo | null>(setup.game ?? setup.found[0] ?? null)
  const [error, setError] = useState<string | null>(null)
  const [installed, setInstalled] = useState(false)
  const running = useGameRunning()
  const installing = useInstall(game?.dir, () => setInstalled(true))
  const note = installing.report && installNote(installing.report, w)
  const pick = () => {
    setError(null)
    chooseGameFolder(s.picker).then(
      (chosen) => chosen && setGame(chosen),
      (e: unknown) => setError(String(e)),
    )
  }
  return (
    <main className="tt tt-first">
      <img src="/logo.svg" alt="" />
      <h1>{s.welcome}</h1>
      <p>{s.text}</p>
      <div className="tt-first-game">
        <Row title={game ? s.found : s.none} detail={game?.dir}>
          <button type="button" className="tt-button quiet" onClick={pick}>
            {s.pick}
          </button>
        </Row>
      </div>
      {running && <p className="tt-note">{w.install.closeGame}</p>}
      {(note ?? error ?? installing.error) && <p className="tt-note bad">{note ?? error ?? installing.error}</p>}
      {installed ? (
        <button type="button" className="tt-button big" onClick={() => void finishSetup().then(done)}>
          {s.finish}
        </button>
      ) : (
        <button
          type="button"
          className="tt-button big"
          disabled={!game || running !== false || installing.working}
          onClick={() => void installing.install()}
        >
          {installing.working ? s.working : installing.administrator ? w.install.asAdministrator : s.install}
        </button>
      )}
      <i className="tt-road" />
    </main>
  )
}

