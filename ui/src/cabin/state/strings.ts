import type { ReturnKind } from '../keyboard/layouts'
import { WEB_APP_NAMES } from '../apps/store/webApps'
import type { AppId } from './apps'

/**
 * The TV's own words, in English and Turkish: Turkish when the game plays in Turkish, English otherwise.
 */

export type Language = 'en' | 'tr'

export interface Strings {
  minutes: (minutes: number) => string
  hours: (hours: number, minutes: string) => string
  /** Turn-by-turn. */
  now: string
  then: string
  zoomIn: string
  zoomOut: string
  /** Maps' search: its button, and the field's hint. */
  search: string
  close: string
  /** The keyboard: its keys (named for screen readers where they show a symbol), and a field's clear button. */
  keyboard: {
    space: string
    returnKey: Record<ReturnKind, string>
    shift: string
    delete: string
    nextKeyboard: string
    hide: string
    clear: string
    /** Whether the player's own keyboard types into the field. */
    typingWithKeyboard: string
    pausedWhileDriving: string
    selectToType: string
  }
  /** Music. */
  nowPlaying: string
  sources: string
  previous: string
  play: string
  pause: string
  next: string
  shuffle: string
  repeat: string
  playing: string
  /** The YouTube app: the search field's hint, Home's sections and chips (each a search), what a list says
   * when it's empty or YouTube didn't answer, the live badge, and the player's words. */
  /** The Spotify app, in Spotify's own words: while its sign-in window is open on the PC; its tabs and the
   * search pill's hint; Home's filters; what things are ("Playlist • Spotify"); the pages' words; Now
   * Playing's; and the toasts of adding to Liked Songs or Your Library. */
  spotify: {
    finishOnPc: string
    whatToPlay: string
    home: string
    library: string
    browse: string
    back: string
    filters: { all: string; music: string; podcasts: string }
    showAll: string
    browseAll: string
    recentSearches: string
    clearRecent: string
    forget: string
    noResults: (query: string) => string
    noResultsHint: string
    topResult: string
    songs: string
    artists: string
    playlists: string
    albums: string
    podcasts: string
    kinds: { playlist: string; album: string; artist: string; show: string; liked: string; song: string }
    songCount: (count: number) => string
    /** A playlist's length: "2 hr 45 min", "38 min 12 sec". */
    length: (seconds: number) => string
    recents: string
    gridView: string
    listView: string
    verified: string
    follow: string
    following: string
    monthlyListeners: (count: string) => string
    popular: string
    discography: string
    playingFrom: { playlist: string; album: string; artist: string; show: string; liked: string }
    nowPlaying: string
    queue: string
    nextFrom: (name: string) => string
    nextUp: string
    /** Now Playing's words for the song (Spotify's Lyrics): none, not timed yet, and where they're from. */
    lyrics: string
    noLyrics: string
    notSynced: string
    lyricsFrom: (provider: string) => string
    /** Now Playing's volume. */
    volume: string
    mute: string
    unmute: string
    /** Spotify didn't play what was asked for. */
    cantPlay: string
    /** Spotify didn't answer. */
    failed: string
    tryAgain: string
    next: string
    previous: string
    shuffle: string
    repeat: string
    close: string
    more: string
    download: string
    addToLiked: string
    addToLibrary: string
    added: string
    removed: string
    addedToLibrary: string
    removedFromLibrary: string
    /** Signing in (Spotify's "Log in"), up and out; the account's button. */
    logIn: string
    signUp: string
    logOut: string
    account: string
    /** Signed out, at the foot: Spotify's banner for its preview. */
    previewTitle: string
    previewText: string
    signUpFree: string
    /** The log-in prompt, by what the player tried: to play, to save, to make a playlist. */
    prompts: {
      play: { title: string; text: string }
      save: { title: string; text: string }
      playlist: { title: string; text: string }
    }
    haveAccount: string
    notNow: string
    /** Your Library, signed out: Spotify's two first steps. */
    firstPlaylist: { title: string; text: string; button: string }
    firstPodcasts: { title: string; text: string; button: string }
    /** The "E" of an explicit song. */
    explicit: string
  }
  /** YouTube Music: TruckersTV's own app, in the look of YouTube Music's tablet app. */
  youtubeMusic: {
    tabs: { home: string; explore: string; library: string; search: string }
    searchHint: string
    remove: string
    more: string
    play: string
    shuffle: string
    radio: string
    save: string
    saved: string
    removed: string
    inLibrary: string
    like: string
    dislike: string
    /** The Song/Video switch. */
    song: string
    video: string
    upNext: string
    lyrics: string
    related: string
    playingFrom: string
    noLyrics: string
    shelves: {
      playlists: string
      songs: string
      albums: string
      artists: string
      subscriptions: string
      history: string
    }
    topResult: string
    /** "1.2M subscribers" */
    subscribers: (count: string) => string
    signIn: string
    signOut: string
    account: string
    /** While Google's sign-in window is open on the PC. */
    finishOnPc: string
    /** The Library, signed out. */
    libraryPitch: { title: string; text: string }
    /** The sign-in prompt, and why. */
    promptTitle: string
    promptFor: { library: string; like: string; save: string }
    notNow: string
    empty: string
    noResults: string
    cantPlay: string
    failed: string
    tryAgain: string
    back: string
    close: string
    nowPlaying: string
    pause: string
    next: string
    previous: string
    repeat: string
    /** An ad playing (YouTube Music's own, if one ever comes through). */
    ad: string
    /** A song's ⋮ and its choices, and what's said as they're made. */
    moreActions: string
    startRadio: string
    playNext: string
    addToQueue: string
    goToAlbum: string
    goToArtist: string
    willPlayNext: string
    addedToQueue: string
    volume: string
  }
  /** Chrome: its toolbar and what it says. */
  chrome: {
    back: string
    forward: string
    reload: string
    stop: string
    address: string
    loading: string
    zoomIn: string
    zoomOut: string
    restart: string
    /** Outside TruckersTV (the page in a desktop browser), where no website can show. */
    outsideHost: string
  }
  /** TV: the player's M3U playlists, their channels and the channel playing. */
  tv: {
    fullScreen: string
    /** Under Loading, for a list or a channel that doesn't come. */
    loadingHint: string
    exitFullScreen: string
    /** The profiles, as Netflix's: its title, the card adding one, and managing (removing) them. */
    sources: string
    addProfile: string
    manageProfiles: string
    done: string
    /** Adding a profile: what it plays, each kind's name and what it is. */
    chooseKind: string
    kinds: Record<'m3u' | 'xtream' | 'videos', string>
    kindAbout: Record<'m3u' | 'xtream' | 'videos', string>
    /** The form's fields and buttons. */
    name: string
    link: string
    linkHint: string
    server: string
    serverHint: string
    username: string
    password: string
    save: string
    paste: string
    back: string
    remove: string
    /** A free list of channels (iptv-org), offered in M3U's form. */
    suggested: string
    search: string
    all: string
    loading: string
    noChannels: string
    playlistFailed: string
    xtreamFailed: string
    /** PC videos with no folder chosen: where to choose it. */
    noVideoFolder: string
    noVideos: string
    streamFailed: string
    /** A channel with no name of its own. */
    live: string
    play: string
    pause: string
    stop: string
    volumeUp: string
    volumeDown: string
  }
  /** The Settings app: its pages, rows and the words under them. */
  settings: {
    light: string
    dark: string
    textSize: string
    /** Text Size's own text, shown at the size chosen (iOS's). */
    textSizeAbout: string
    /** Sound, and in it the Equalizer: the speakers as tuned (Recommended) or the player's own tone (Custom), what
     * it does, its three bands under the graph, and the curve set back. */
    sound: string
    equalizer: string
    recommended: string
    custom: string
    equalizerFooter: string
    low: string
    mid: string
    high: string
    resetEqualizer: string
    back: string
  }
  /** Apps. */
  /** The App Store: its sites' apps, got onto Home, opened, or taken off. */
  launcher: {
    home: string
    apps: string
    library: string
    continue: string
    signIn: string
    signingIn: string
    arrange: string
    arrangeHint: string
    arrangeDone: string
    recent: string
    yourApps: string
    search: string
    searchHint: string
    noApps: string
    categories: string
    categoryNames: { entertainment: string; music: string; video: string; sports: string; tools: string }
    sources: string
    addSource: string
  }
  apps: Record<AppId, string>
}

export const STRINGS: Record<Language, Strings> = {
  en: {
    minutes: (minutes) => `${minutes} min`,
    hours: (hours, minutes) => `${hours}:${minutes} hrs`,
    now: 'Now',
    then: 'Then',
    zoomIn: 'Zoom in',
    zoomOut: 'Zoom out',
    search: 'Search',
    close: 'Close',
    keyboard: {
      space: 'space',
      returnKey: { search: 'Search', go: 'Go', done: 'Done' },
      shift: 'Shift',
      delete: 'Delete',
      nextKeyboard: 'Next keyboard',
      hide: 'Hide keyboard',
      clear: 'Clear text',
      typingWithKeyboard: 'Typing with your keyboard',
      pausedWhileDriving: 'Keyboard paused while driving',
      selectToType: 'Select the field to type with your keyboard',
    },
    nowPlaying: 'Now Playing',
    sources: 'Sources',
    previous: 'Previous',
    play: 'Play',
    pause: 'Pause',
    next: 'Next',
    shuffle: 'Shuffle',
    repeat: 'Repeat',
    playing: 'Playing',
    spotify: {
      podcasts: 'Podcasts & Shows',
      finishOnPc: 'Finish signing in on your PC: Spotify’s sign-in is open in a window on your desktop.',
      whatToPlay: 'What do you want to play?',
      home: 'Home',
      library: 'Your Library',
      browse: 'Browse',
      back: 'Back',
      filters: { all: 'All', music: 'Music', podcasts: 'Podcasts' },
      showAll: 'Show all',
      browseAll: 'Browse all',
      recentSearches: 'Recent searches',
      clearRecent: 'Clear recent searches',
      forget: 'Remove',
      noResults: (query) => `No results found for “${query}”`,
      noResultsHint: 'Check the spelling, or try fewer or different words.',
      topResult: 'Top result',
      songs: 'Songs',
      artists: 'Artists',
      playlists: 'Playlists',
      albums: 'Albums',
      kinds: {
        playlist: 'Playlist',
        album: 'Album',
        artist: 'Artist',
        show: 'Podcast',
        liked: 'Playlist',
        song: 'Song',
      },
      songCount: (count) => (count === 1 ? '1 song' : `${count} songs`),
      length: (seconds) => {
        const hours = Math.floor(seconds / 3600)
        const minutes = Math.floor((seconds % 3600) / 60)
        return hours > 0 ? `${hours} hr ${minutes} min` : `${minutes} min ${Math.floor(seconds % 60)} sec`
      },
      recents: 'Recents',
      gridView: 'Grid view',
      listView: 'List view',
      verified: 'Verified Artist',
      follow: 'Follow',
      following: 'Following',
      monthlyListeners: (count) => `${count} monthly listeners`,
      popular: 'Popular',
      discography: 'Discography',
      playingFrom: {
        playlist: 'Playing from playlist',
        album: 'Playing from album',
        artist: 'Playing from artist',
        show: 'Playing from podcast',
        liked: 'Playing from your library',
      },
      nowPlaying: 'Now playing',
      queue: 'Queue',
      nextFrom: (name) => `Next from: ${name}`,
      nextUp: 'Next up',
      lyrics: 'Lyrics',
      noLyrics: 'Looks like we don’t have the lyrics for this song.',
      notSynced: 'These lyrics aren’t synced to the song yet.',
      lyricsFrom: (provider) => `Lyrics provided by ${provider}`,
      volume: 'Volume',
      mute: 'Mute',
      unmute: 'Unmute',
      cantPlay: 'Spotify can’t play this right now.',
      failed: 'Something went wrong',
      tryAgain: 'Try again',
      next: 'Next',
      previous: 'Previous',
      shuffle: 'Shuffle',
      repeat: 'Repeat',
      close: 'Close',
      more: 'More options',
      download: 'Download',
      addToLiked: 'Add to Liked Songs',
      addToLibrary: 'Add to Your Library',
      added: 'Added to Liked Songs.',
      removed: 'Removed from Liked Songs.',
      addedToLibrary: 'Added to Your Library.',
      removedFromLibrary: 'Removed from Your Library.',
      logIn: 'Log in',
      signUp: 'Sign up',
      logOut: 'Log out',
      account: 'Account',
      previewTitle: 'Preview of Spotify',
      previewText: 'Sign up to get unlimited songs and podcasts with occasional ads. No credit card needed.',
      signUpFree: 'Sign up free',
      prompts: {
        play: { title: 'Start listening with a free Spotify account', text: '' },
        save: { title: 'Save to Your Library', text: 'Log in to save songs, albums, playlists and artists.' },
        playlist: { title: 'Create a playlist', text: 'Log in to create and share playlists.' },
      },
      haveAccount: 'Already have an account?',
      notNow: 'Not now',
      firstPlaylist: {
        title: 'Create your first playlist',
        text: 'It’s easy, we’ll help you',
        button: 'Create playlist',
      },
      firstPodcasts: {
        title: 'Let’s find some podcasts to follow',
        text: 'We’ll keep you updated on new episodes',
        button: 'Browse podcasts',
      },
      explicit: 'E',
    },
    youtubeMusic: {
      tabs: { home: 'Home', explore: 'Explore', library: 'Library', search: 'Search' },
      searchHint: 'Search songs, artists, podcasts',
      remove: 'Remove',
      more: 'More',
      play: 'Play',
      shuffle: 'Shuffle',
      radio: 'Radio',
      save: 'Save to library',
      saved: 'Saved to library',
      removed: 'Removed from library',
      inLibrary: 'Saved',
      like: 'Like',
      dislike: 'Dislike',
      song: 'Song',
      video: 'Video',
      upNext: 'Up next',
      lyrics: 'Lyrics',
      related: 'Related',
      playingFrom: 'Playing from',
      noLyrics: 'Lyrics aren’t available for this song.',
      shelves: {
        playlists: 'Playlists',
        songs: 'Songs',
        albums: 'Albums',
        artists: 'Artists',
        subscriptions: 'Subscriptions',
        history: 'History',
      },
      topResult: 'Top result',
      subscribers: (count) => `${count} subscribers`,
      signIn: 'Sign in',
      signOut: 'Sign out',
      account: 'Account',
      finishOnPc: 'Finish signing in on your PC.',
      libraryPitch: {
        title: 'Enjoy your favourite music',
        text: 'Sign in to see your playlists, liked songs and albums. Your YouTube sign-in works here too.',
      },
      promptTitle: 'Sign in to YouTube Music',
      promptFor: {
        library: 'Your playlists, songs and albums are in your library.',
        like: 'Songs you like go to your Liked music.',
        save: 'Albums and playlists you save go to your library.',
      },
      notNow: 'Not now',
      empty: 'Nothing here yet.',
      noResults: 'No results',
      cantPlay: 'YouTube Music can’t play this right now.',
      failed: 'Couldn’t load this.',
      tryAgain: 'Try again',
      back: 'Back',
      close: 'Close',
      nowPlaying: 'Now playing',
      pause: 'Pause',
      next: 'Next',
      previous: 'Previous',
      repeat: 'Repeat',
      ad: 'Ad',
      moreActions: 'More actions',
      startRadio: 'Start radio',
      playNext: 'Play next',
      addToQueue: 'Add to queue',
      goToAlbum: 'Go to album',
      goToArtist: 'Go to artist',
      willPlayNext: 'Song will play next',
      addedToQueue: 'Song added to queue',
      volume: 'Volume',
    },
    chrome: {
      back: 'Back',
      forward: 'Forward',
      reload: 'Reload',
      stop: 'Stop',
      address: 'Search or enter website name',
      loading: 'Loading…',
      zoomIn: 'Zoom In',
      zoomOut: 'Zoom Out',
      restart: 'Restart',
      outsideHost: 'Websites open in the TruckersTV app.',
    },
    tv: {
      fullScreen: 'Full Screen',
      loadingHint: 'If it doesn’t open, leave the app and come back',
      exitFullScreen: 'Exit Full Screen',
      sources: 'Sources',
      addProfile: 'Add Profile',
      manageProfiles: 'Manage Profiles',
      done: 'Done',
      chooseKind: 'What does it play?',
      kinds: { m3u: 'M3U Playlist', xtream: 'Xtream', videos: 'PC Videos' },
      kindAbout: {
        m3u: 'A link to an M3U or M3U8 list of channels',
        xtream: 'Your provider’s server, username and password',
        videos: 'The videos in the folder chosen in TruckersTV on your PC',
      },
      name: 'Profile name',
      link: 'Link',
      linkHint: 'https://…/list.m3u',
      server: 'Server',
      serverHint: 'http://server:8080',
      username: 'Username',
      password: 'Password',
      save: 'Save',
      paste: 'Paste',
      back: 'Back',
      remove: 'Remove',
      suggested: 'Free channels (iptv-org)',
      search: 'Search',
      all: 'All',
      loading: 'Loading…',
      noChannels: 'Nothing here',
      playlistFailed: 'The playlist couldn’t be loaded.',
      xtreamFailed: 'The server didn’t sign in. Check its address, username and password.',
      noVideoFolder: 'Choose a videos folder in TruckersTV on your PC: Customize › TV.',
      noVideos: 'No videos in the folder.',
      streamFailed: 'This couldn’t be played.',
      live: 'Live',
      play: 'Play',
      pause: 'Pause',
      stop: 'Stop',
      volumeUp: 'Volume up',
      volumeDown: 'Volume down',
    },
    settings: {
      light: 'Light',
      dark: 'Dark',
      textSize: 'Text Size',
      textSizeAbout: 'TruckersTV’s apps will adjust to your preferred reading size below.',
      sound: 'Sound',
      equalizer: 'Equalizer',
      recommended: 'Recommended',
      custom: 'Custom',
      equalizerFooter:
        'TruckersTV plays your music through each truck’s own speakers, tuned for its cab. If you prefer a different sound, you can customise how every music app sounds, in every truck.',
      low: 'Low',
      mid: 'Mid',
      high: 'High',
      resetEqualizer: 'Reset',
      back: 'Back',
    },
    launcher: {
      home: 'Home',
      apps: 'Apps',
      library: 'Library',
      continue: 'Continue',
      signIn: 'Sign in',
      signingIn: 'Finish signing in on the PC',
      arrange: 'Arrange apps',
      arrangeHint: 'Press an app, then press where it goes',
      arrangeDone: 'Done',
      recent: 'Recent',
      yourApps: 'Your apps',
      search: 'Search',
      searchHint: 'Search for apps',
      noApps: 'No apps found',
      categories: 'App categories',
      categoryNames: {
        entertainment: 'Entertainment',
        music: 'Music & Audio',
        video: 'Video Players',
        sports: 'Sports',
        tools: 'Tools',
      },
      sources: 'Media sources',
      addSource: 'Add source',
    },
    apps: {
      ...WEB_APP_NAMES,
      spotify: 'Spotify',
      youtube: 'YouTube',
      youtubeMusic: 'YouTube Music',
      chrome: 'Chrome',
      tv: 'Live TV',
      settings: 'Settings',
    },
  },
  tr: {
    minutes: (minutes) => `${minutes} dk`,
    hours: (hours, minutes) => `${hours}:${minutes} sa`,
    now: 'Şimdi',
    then: 'Sonra',
    zoomIn: 'Yakınlaştır',
    zoomOut: 'Uzaklaştır',
    search: 'Ara',
    close: 'Kapat',
    keyboard: {
      space: 'boşluk',
      returnKey: { search: 'Ara', go: 'Git', done: 'Bitti' },
      shift: 'Büyük harf',
      delete: 'Sil',
      nextKeyboard: 'Sonraki klavye',
      hide: 'Klavyeyi gizle',
      clear: 'Metni temizle',
      typingWithKeyboard: 'Klavyenizle yazıyorsunuz',
      pausedWhileDriving: 'Sürüş sırasında klavye duraklatıldı',
      selectToType: 'Klavyenizle yazmak için alanı seçin',
    },
    nowPlaying: 'Şimdi Çalıyor',
    sources: 'Kaynaklar',
    previous: 'Önceki',
    play: 'Oynat',
    pause: 'Duraklat',
    next: 'Sonraki',
    shuffle: 'Karıştır',
    repeat: 'Tekrarla',
    playing: 'Çalıyor',
    spotify: {
      podcasts: 'Podcast’ler ve Programlar',
      finishOnPc: 'Oturum açmayı bilgisayarınızda tamamlayın: Spotify’ın oturum açma penceresi masaüstünüzde açık.',
      whatToPlay: 'Ne çalmak istiyorsun?',
      home: 'Ana sayfa',
      library: 'Kitaplığın',
      browse: 'Göz at',
      back: 'Geri',
      filters: { all: 'Tümü', music: 'Müzik', podcasts: 'Podcast’ler' },
      showAll: 'Tümünü göster',
      browseAll: 'Hepsine göz at',
      recentSearches: 'Son aramalar',
      clearRecent: 'Son aramaları temizle',
      forget: 'Kaldır',
      noResults: (query) => `“${query}” için sonuç bulunamadı`,
      noResultsHint: 'Yazımı kontrol et ya da daha az veya farklı kelimeler dene.',
      topResult: 'En iyi sonuç',
      songs: 'Şarkılar',
      artists: 'Sanatçılar',
      playlists: 'Çalma listeleri',
      albums: 'Albümler',
      kinds: {
        playlist: 'Çalma listesi',
        album: 'Albüm',
        artist: 'Sanatçı',
        show: 'Podcast',
        liked: 'Çalma listesi',
        song: 'Şarkı',
      },
      songCount: (count) => `${count} şarkı`,
      length: (seconds) => {
        const hours = Math.floor(seconds / 3600)
        const minutes = Math.floor((seconds % 3600) / 60)
        return hours > 0 ? `${hours} sa. ${minutes} dk.` : `${minutes} dk. ${Math.floor(seconds % 60)} sn.`
      },
      recents: 'En son',
      gridView: 'Izgara görünümü',
      listView: 'Liste görünümü',
      verified: 'Onaylanmış Sanatçı',
      follow: 'Takip et',
      following: 'Takip ediliyor',
      monthlyListeners: (count) => `${count} aylık dinleyici`,
      popular: 'Popüler',
      discography: 'Diskografi',
      playingFrom: {
        playlist: 'Çalma listesinden çalınıyor',
        album: 'Albümden çalınıyor',
        artist: 'Sanatçıdan çalınıyor',
        show: 'Podcast’ten çalınıyor',
        liked: 'Kitaplığından çalınıyor',
      },
      nowPlaying: 'Şu anda çalan',
      queue: 'Sıra',
      nextFrom: (name) => `Sıradaki: ${name}`,
      nextUp: 'Sıradakiler',
      lyrics: 'Şarkı sözleri',
      noLyrics: 'Görünüşe göre bu şarkının sözleri bizde yok.',
      notSynced: 'Bu sözler henüz şarkıyla eşzamanlı değil.',
      lyricsFrom: (provider) => `Şarkı sözlerini sağlayan: ${provider}`,
      volume: 'Ses düzeyi',
      mute: 'Sesi kapat',
      unmute: 'Sesi aç',
      cantPlay: 'Spotify bunu şu anda çalamıyor.',
      failed: 'Bir şeyler ters gitti',
      tryAgain: 'Tekrar dene',
      next: 'Sonraki',
      previous: 'Önceki',
      shuffle: 'Karıştır',
      repeat: 'Tekrarla',
      close: 'Kapat',
      more: 'Diğer seçenekler',
      download: 'İndir',
      addToLiked: 'Beğenilen Şarkılar’a ekle',
      addToLibrary: 'Kitaplığına ekle',
      added: 'Beğenilen Şarkılar’a eklendi.',
      removed: 'Beğenilen Şarkılar’dan kaldırıldı.',
      addedToLibrary: 'Kitaplığına eklendi.',
      removedFromLibrary: 'Kitaplığından kaldırıldı.',
      logIn: 'Oturum aç',
      signUp: 'Kaydol',
      logOut: 'Oturumu kapat',
      account: 'Hesap',
      previewTitle: 'Spotify önizlemesi',
      previewText: 'Ara sıra reklamlarla sınırsız şarkı ve podcast dinlemek için kaydol. Kredi kartı gerekmez.',
      signUpFree: 'Ücretsiz kaydol',
      prompts: {
        play: { title: 'Ücretsiz bir Spotify hesabıyla dinlemeye başla', text: '' },
        save: {
          title: 'Kitaplığına kaydet',
          text: 'Şarkıları, albümleri, çalma listelerini ve sanatçıları kaydetmek için oturum aç.',
        },
        playlist: { title: 'Çalma listesi oluştur', text: 'Çalma listesi oluşturmak ve paylaşmak için oturum aç.' },
      },
      haveAccount: 'Zaten bir hesabın var mı?',
      notNow: 'Şimdi değil',
      firstPlaylist: {
        title: 'İlk çalma listeni oluştur',
        text: 'Çok kolay, sana yardım edeceğiz',
        button: 'Çalma listesi oluştur',
      },
      firstPodcasts: {
        title: 'Takip edeceğin podcast’leri bulalım',
        text: 'Yeni bölümlerden seni haberdar edeceğiz',
        button: 'Podcast’lere göz at',
      },
      explicit: 'E',
    },
    youtubeMusic: {
      tabs: { home: 'Ana Sayfa', explore: 'Keşfet', library: 'Kitaplık', search: 'Ara' },
      searchHint: 'Şarkı, sanatçı, podcast ara',
      remove: 'Kaldır',
      more: 'Daha fazla',
      play: 'Oynat',
      shuffle: 'Karıştır',
      radio: 'Radyo',
      save: 'Kitaplığa kaydet',
      saved: 'Kitaplığa kaydedildi',
      removed: 'Kitaplıktan kaldırıldı',
      inLibrary: 'Kaydedildi',
      like: 'Beğen',
      dislike: 'Beğenme',
      song: 'Şarkı',
      video: 'Video',
      upNext: 'Sıradaki',
      lyrics: 'Şarkı sözleri',
      related: 'İlgili',
      playingFrom: 'Şuradan çalınıyor',
      noLyrics: 'Bu şarkının sözleri yok.',
      shelves: {
        playlists: 'Çalma listeleri',
        songs: 'Şarkılar',
        albums: 'Albümler',
        artists: 'Sanatçılar',
        subscriptions: 'Abonelikler',
        history: 'Geçmiş',
      },
      topResult: 'En iyi sonuç',
      subscribers: (count) => `${count} abone`,
      signIn: 'Oturum aç',
      signOut: 'Oturumu kapat',
      account: 'Hesap',
      finishOnPc: 'Oturum açmayı bilgisayarınızda tamamlayın.',
      libraryPitch: {
        title: 'Sevdiğiniz müziğin keyfini çıkarın',
        text: 'Çalma listelerinizi, beğendiğiniz şarkıları ve albümlerinizi görmek için oturum açın. YouTube oturumunuz burada da geçerli.',
      },
      promptTitle: 'YouTube Music’te oturum açın',
      promptFor: {
        library: 'Çalma listeleriniz, şarkılarınız ve albümleriniz kitaplığınızda.',
        like: 'Beğendiğiniz şarkılar Beğenilen müzikler listenize eklenir.',
        save: 'Kaydettiğiniz albümler ve çalma listeleri kitaplığınıza eklenir.',
      },
      notNow: 'Şimdi değil',
      empty: 'Burada henüz bir şey yok.',
      noResults: 'Sonuç yok',
      cantPlay: 'YouTube Music bunu şu anda çalamıyor.',
      failed: 'Yüklenemedi.',
      tryAgain: 'Tekrar dene',
      back: 'Geri',
      close: 'Kapat',
      nowPlaying: 'Şimdi çalıyor',
      pause: 'Duraklat',
      next: 'Sonraki',
      previous: 'Önceki',
      repeat: 'Tekrarla',
      ad: 'Reklam',
      moreActions: 'Diğer işlemler',
      startRadio: 'Radyoyu başlat',
      playNext: 'Sıradaki olarak çal',
      addToQueue: 'Sıraya ekle',
      goToAlbum: 'Albüme git',
      goToArtist: 'Sanatçıya git',
      willPlayNext: 'Şarkı sıradaki olarak çalınacak',
      addedToQueue: 'Şarkı sıraya eklendi',
      volume: 'Ses',
    },
    chrome: {
      back: 'Geri',
      forward: 'İleri',
      reload: 'Yenile',
      stop: 'Durdur',
      address: 'Ara veya web sitesi adı gir',
      loading: 'Yükleniyor…',
      zoomIn: 'Yakınlaştır',
      zoomOut: 'Uzaklaştır',
      restart: 'Yeniden başlat',
      outsideHost: 'Web siteleri TruckersTV uygulamasında açılır.',
    },
    tv: {
      fullScreen: 'Tam Ekran',
      loadingHint: 'Eğer açılmazsa uygulamadan çıkıp tekrar girin',
      exitFullScreen: 'Tam ekrandan çık',
      sources: 'Kaynaklar',
      addProfile: 'Profil Ekle',
      manageProfiles: 'Profilleri Yönet',
      done: 'Bitti',
      chooseKind: 'Ne oynatacak?',
      kinds: { m3u: 'M3U Listesi', xtream: 'Xtream', videos: 'PC Videoları' },
      kindAbout: {
        m3u: 'Kanalların M3U veya M3U8 listesinin linki',
        xtream: 'Sağlayıcının sunucusu, kullanıcı adı ve şifresi',
        videos: 'PC’deki TruckersTV’de seçilen klasördeki videolar',
      },
      name: 'Profil adı',
      link: 'Link',
      linkHint: 'https://…/liste.m3u',
      server: 'Sunucu',
      serverHint: 'http://sunucu:8080',
      username: 'Kullanıcı adı',
      password: 'Şifre',
      save: 'Kaydet',
      paste: 'Yapıştır',
      back: 'Geri',
      remove: 'Sil',
      suggested: 'Ücretsiz kanallar (iptv-org)',
      search: 'Ara',
      all: 'Tümü',
      loading: 'Yükleniyor…',
      noChannels: 'Burada bir şey yok',
      playlistFailed: 'Liste yüklenemedi.',
      xtreamFailed: 'Sunucuya giriş yapılamadı. Adresi, kullanıcı adını ve şifreyi kontrol et.',
      noVideoFolder: 'PC’deki TruckersTV’de bir video klasörü seç: Özelleştir › TV.',
      noVideos: 'Klasörde video yok.',
      streamFailed: 'Bu oynatılamadı.',
      live: 'Canlı',
      play: 'Oynat',
      pause: 'Duraklat',
      stop: 'Durdur',
      volumeUp: 'Sesi aç',
      volumeDown: 'Sesi kıs',
    },
    settings: {
      light: 'Açık',
      dark: 'Koyu',
      textSize: 'Metin Boyutu',
      textSizeAbout: 'TruckersTV’in uygulamaları aşağıda seçtiğin okuma boyutuna göre ayarlanır.',
      sound: 'Ses',
      equalizer: 'Ekolayzır',
      recommended: 'Önerilen',
      custom: 'Özel',
      equalizerFooter:
        'TruckersTV müziğini her kamyonun kendi hoparlörlerinden, kabinine göre ayarlanmış olarak çalar. Farklı bir ses istersen, her müzik uygulamasının her kamyonda nasıl çalacağını özelleştirebilirsin.',
      low: 'Düşük',
      mid: 'Orta',
      high: 'Yüksek',
      resetEqualizer: 'Sıfırla',
      back: 'Geri',
    },
    launcher: {
      home: 'Ana Sayfa',
      apps: 'Uygulamalar',
      library: 'Kitaplık',
      continue: 'Devam et',
      signIn: 'Giriş yap',
      signingIn: 'PC’de giriş yapın',
      arrange: 'Uygulamaları sırala',
      arrangeHint: 'Bir uygulamaya basın, sonra gideceği yere basın',
      arrangeDone: 'Bitti',
      recent: 'Son kullanılanlar',
      yourApps: 'Uygulamalarınız',
      search: 'Ara',
      searchHint: 'Uygulama ara',
      noApps: 'Uygulama bulunamadı',
      categories: 'Uygulama kategorileri',
      categoryNames: {
        entertainment: 'Eğlence',
        music: 'Müzik ve Ses',
        video: 'Video Oynatıcılar',
        sports: 'Spor',
        tools: 'Araçlar',
      },
      sources: 'Medya kaynakları',
      addSource: 'Kaynak ekle',
    },
    apps: {
      ...WEB_APP_NAMES,
      spotify: 'Spotify',
      youtube: 'YouTube',
      youtubeMusic: 'YouTube Music',
      chrome: 'Chrome',
      tv: 'CanlıTV',
      settings: 'Ayarlar',
    },
  },
}

/** The cabin's language for the game's (its `g_lang_init`, e.g. `tr_tr`; `null` until known). */
export function languageOf(gameLanguage: string | null): Language {
  return gameLanguage?.toLowerCase().startsWith('tr') ? 'tr' : 'en'
}
