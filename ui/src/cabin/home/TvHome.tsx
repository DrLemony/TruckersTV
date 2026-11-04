import { type CSSProperties, type ReactNode, useState } from 'react'
import { formatGameTime } from '../../shared/format'
import { cabClock } from '../../shared/host'
import { appIcon } from '../../shared/icons'
import { isWebAppId, webApp } from '../apps/store/webApps'
import { WebAppIcon } from '../apps/store/WebAppIcon'
import { useTv } from '../apps/tv/player'
import { loadProfiles, type Profile } from '../apps/tv/profiles'
import { TextField } from '../keyboard/TextField'
import { useKeyboard } from '../keyboard/store'
import { APP_ICONS, type AppId } from '../state/apps'
import { useLaunchStyle } from '../state/launch'
import { go, useCabin, useLive } from '../state/store'
import { useText } from '../state/text'
import { useHomeOrder } from './order'
import './tvhome.css'

const CATEGORIES = {
  entertainment: ['exxen', 'netflix', 'prime', 'disney', 'tvplus'],
  music: ['youtubeMusic', 'spotify'],
  video: ['youtube', 'tv'],
  sports: ['tod', 'exxen'],
  tools: ['chrome'],
} satisfies Record<string, AppId[]>

type Category = keyof typeof CATEGORIES

const CATEGORY_NAMES = Object.keys(CATEGORIES) as Category[]

/** What's behind each app's icon, and the colour it glows under the cursor. */
const LOOKS: Partial<Record<AppId, { back: string; glow: string }>> = {
  youtube: { back: '#fff', glow: '#ff0033' },
  youtubeMusic: { back: '#fff', glow: '#ff0033' },
  spotify: { back: '#000', glow: '#1ed760' },
  exxen: { back: '#ffe11b', glow: '#ffe11b' },
  netflix: { back: '#f5f5f5', glow: '#e50914' },
  prime: { back: '#0578ff', glow: '#0578ff' },
  disney: { back: 'linear-gradient(#081e2b, #2a7d88)', glow: '#2a9d9f' },
  tod: { back: '#000', glow: '#f5b301' },
  tvplus: { back: '#0d3b66', glow: '#2f80ed' },
  tv: { back: 'linear-gradient(#464646, #141414)', glow: '#8ab4f8' },
  chrome: { back: '#fff', glow: '#4285f4' },
}

const lookOf = (app: AppId) =>
  ({
    '--disc-back': LOOKS[app]?.back ?? '#2c2c30',
    '--disc-glow': LOOKS[app]?.glow ?? '#8ab4f8',
  }) as CSSProperties

/** The sites whose icon has a background of its own: it fills the disc. */
const FILLS_DISC: ReadonlySet<AppId> = new Set<AppId>(['exxen', 'prime', 'disney', 'tvplus'])

const TABS = ['home', 'apps', 'library'] as const

type Tab = (typeof TABS)[number]

// The tab left, for coming back from an app.
const left: { tab: Tab } = { tab: 'apps' }

const SEARCH_ID = 'home-search'

/** An app's round icon. */
function Disc({ app, style }: { app: AppId; style?: CSSProperties }) {
  const role = APP_ICONS[app]
  return (
    <span className="tv-disc" style={{ ...lookOf(app), ...style }}>
      {role ? (
        <img src={appIcon(role)} alt="" className="tv-disc-icon" />
      ) : (
        isWebAppId(app) && <WebAppIcon app={webApp(app)} className={FILLS_DISC.has(app) ? 'tv-disc-icon web fills' : 'tv-disc-icon web'} />
      )}
    </span>
  )
}

/**
 * The TV's home screen, after Google TV's: the tabs and the round buttons on top, and under them the last app
 * used (Home), the apps with their search and categories (Apps), or what Live TV has (Library).
 */
export function TvHome() {
  const text = useText()
  const [tab, setTab] = useState(left.tab)
  // How many times the magnifier was pressed: each brings the keyboard up on the search field.
  const [searches, setSearches] = useState(0)
  const gameTime = useLive((state) => (state.telemetry ? cabClock(state.telemetry) : null))
  const choose = (next: Tab) => {
    left.tab = next
    setTab(next)
  }

  return (
    <section className="screen tv-home" aria-label="Home">
      <header className="tv-home-top">
        <span className="tv-home-avatar" aria-hidden>
          T
        </span>
        <nav className="tv-home-tabs">
          {TABS.map((name) => (
            <button
              key={name}
              type="button"
              className={name === tab ? 'chosen' : undefined}
              onClick={() => choose(name)}
            >
              {text.launcher[name]}
            </button>
          ))}
        </nav>
        <button
          type="button"
          className="tv-home-round"
          aria-label={text.launcher.search}
          onClick={() => {
            choose('apps')
            setSearches(searches + 1)
          }}
        >
          <svg viewBox="0 0 24 24" aria-hidden>
            <circle cx="10.5" cy="10.5" r="6" fill="none" stroke="currentColor" strokeWidth="2.4" />
            <path d="m15 15 5.5 5.5" stroke="currentColor" strokeWidth="2.6" strokeLinecap="round" />
          </svg>
        </button>
        <button
          type="button"
          className="tv-home-round"
          data-app="settings"
          aria-label={text.apps.settings}
          onClick={() => go({ type: 'openApp', app: 'settings' }, 'home')}
        >
          <svg viewBox="0 0 24 24" aria-hidden>
            <path
              fill="currentColor"
              fillRule="evenodd"
              d="M19.4 13a7.6 7.6 0 0 0 0-2l2-1.6-2-3.4-2.4 1a7.5 7.5 0 0 0-1.7-1L15 3.5h-6l-.4 2.5a7.5 7.5 0 0 0-1.7 1l-2.4-1-2 3.4L4.6 11a7.6 7.6 0 0 0 0 2l-2 1.6 2 3.4 2.4-1a7.5 7.5 0 0 0 1.7 1l.4 2.5h6l.4-2.5a7.5 7.5 0 0 0 1.7-1l2.4 1 2-3.4zM12 15.2a3.2 3.2 0 1 1 0-6.4 3.2 3.2 0 0 1 0 6.4z"
            />
          </svg>
        </button>
        <span className="tv-home-brand">TruckersTV</span>
        <span className="tv-home-time">{gameTime === null ? '--:--' : formatGameTime(gameTime)}</span>
      </header>
      {tab === 'home' && <HomeTab />}
      {tab === 'apps' && <AppsTab searches={searches} />}
      {tab === 'library' && <LibraryTab />}
    </section>
  )
}

/** The app last used large, and the ones used lately. */
function HomeTab() {
  const text = useText().launcher
  const recents = useCabin((state) => state.navigation.recents)
  const order = useHomeOrder((state) => state.order)
  const used = recents.filter((app) => order.includes(app))
  return (
    <>
      <Hero app={used[0] ?? order[0]} />
      <h2>{text.recent}</h2>
      <AppRow apps={used.length > 0 ? used : order} />
    </>
  )
}

/** Your apps, narrowed by the search or a category. */
function AppsTab({ searches }: { searches: number }) {
  const text = useText()
  const [query, setQuery] = useState('')
  const [category, setCategory] = useState<Category | null>(null)
  const [arranging, setArranging] = useState(false)
  const order = useHomeOrder((state) => state.order)
  const searching = useKeyboard((state) => state.field?.id === SEARCH_ID)
  const wanted = query.trim().toLocaleLowerCase()
  const apps = order.filter(
    (app) =>
      (!category || (CATEGORIES[category] as AppId[]).includes(app)) &&
      text.apps[app].toLocaleLowerCase().includes(wanted),
  )
  if (arranging) return <Arrange onDone={() => setArranging(false)} />
  return (
    <div className={searching ? 'tv-apps searching' : 'tv-apps'}>
      <h1>{text.launcher.yourApps}</h1>
      {apps.length > 0 ? (
        <AppRow apps={apps}>
          <button type="button" className="tv-tile" onClick={() => setArranging(true)}>
            <span className="tv-disc tv-disc-arrange">
              <svg viewBox="0 0 24 24" aria-hidden>
                <path
                  d="M7 4v14m0 0-3.5-3.5M7 18l3.5-3.5M17 20V6m0 0-3.5 3.5M17 6l3.5 3.5"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </span>
            <span className="tv-tile-name">{text.launcher.arrange}</span>
          </button>
        </AppRow>
      ) : (
        <p className="tv-none">{text.launcher.noApps}</p>
      )}
      <div className="tv-search">
        <TextField
          key={searches}
          id={SEARCH_ID}
          value={query}
          onChange={setQuery}
          hint={text.launcher.searchHint}
          label={text.launcher.search}
          selectOnShow={searches > 0}
          icon={
            <span className="tv-search-dots" aria-hidden>
              <i />
              <i />
              <i />
              <i />
            </span>
          }
        />
      </div>
      <h2>{text.launcher.categories}</h2>
      <div className="tv-chips">
        {CATEGORY_NAMES.map((name) => (
          <button
            key={name}
            type="button"
            className={name === category ? 'chosen' : undefined}
            onClick={() => setCategory(name === category ? null : name)}
          >
            {text.launcher.categoryNames[name]}
          </button>
        ))}
      </div>
    </div>
  )
}

/** Changing the apps' order: an app pressed is picked up, and the next one pressed is where it goes. */
function Arrange({ onDone }: { onDone(): void }) {
  const text = useText()
  const order = useHomeOrder((state) => state.order)
  const move = useHomeOrder((state) => state.move)
  const [held, setHeld] = useState<AppId | null>(null)
  return (
    <div className="tv-arrange">
      <header>
        <div>
          <h1>{text.launcher.arrange}</h1>
          <p>{text.launcher.arrangeHint}</p>
        </div>
        <button type="button" onClick={onDone}>
          {text.launcher.arrangeDone}
        </button>
      </header>
      <div className="tv-arrange-grid">
        {order.map((app) => (
          <button
            key={app}
            type="button"
            className={app === held ? 'tv-tile held' : 'tv-tile'}
            onClick={() => {
              if (held && held !== app) move(held, app)
              setHeld(held ? null : app)
            }}
          >
            <Disc app={app} />
            <span className="tv-tile-name">{text.apps[app]}</span>
          </button>
        ))}
      </div>
    </div>
  )
}

/** Live TV's sources, each opened from here, or the way to add the first. */
function LibraryTab() {
  const text = useText()
  const [sources] = useState(loadProfiles)
  const open = (source: Profile | null) => {
    useTv.setState({ source })
    go({ type: 'openApp', app: 'tv' }, 'home')
  }
  return (
    <>
      <h1 className="tv-library-title">{text.launcher.sources}</h1>
      <div className="tv-chips tv-sources">
        {sources.map((source) => (
          <button key={source.id} type="button" onClick={() => open(source)}>
            <b>{source.name}</b>
            <span>{text.tv.kinds[source.kind]}</span>
          </button>
        ))}
        <button type="button" onClick={() => open(null)}>
          <b>+</b>
          <span>{text.launcher.addSource}</span>
        </button>
      </div>
    </>
  )
}

function AppRow({ apps, children }: { apps: AppId[]; children?: ReactNode }) {
  return (
    <div className="tv-apps-row">
      {apps.map((app) => (
        <TvTile key={app} app={app} />
      ))}
      {children}
    </div>
  )
}

/** The app last used, to go back to. Its tile in the row is what the app grows out of. */
function Hero({ app }: { app: AppId }) {
  const text = useText()
  return (
    <button type="button" className="tv-hero" style={lookOf(app)} onClick={() => go({ type: 'openApp', app }, 'home')}>
      <span className="tv-hero-words">
        <span className="tv-hero-name">{text.apps[app]}</span>
        <span className="tv-hero-go">{text.launcher.continue}</span>
      </span>
      <Disc app={app} />
    </button>
  )
}

/** An app's tile. Its icon is what the app grows out of when it opens, and shrinks back into. */
function TvTile({ app }: { app: AppId }) {
  const launch = useLaunchStyle(app, { kind: 'icon', origin: 'home' })
  const text = useText()
  return (
    <button type="button" className="tv-tile" data-app={app} onClick={() => go({ type: 'openApp', app }, 'home')}>
      <Disc app={app} style={launch} />
      <span className="tv-tile-name">{text.apps[app]}</span>
    </button>
  )
}
