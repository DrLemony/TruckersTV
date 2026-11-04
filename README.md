<p align="center"><img src="app/icons/128x128@2x.png" width="112" alt="TruckersTV"></p>

<h1 align="center">TruckersTV</h1>

<p align="center">
  <b>A television in the cab of your truck in Euro Truck Simulator 2.</b><br>
  An Android TV style home screen with YouTube, music, live TV, a browser and your own videos.
</p>

<p align="center">
  <a href="https://github.com/DrLemony/TruckersTV/releases"><img src="https://img.shields.io/github/v/release/DrLemony/TruckersTV?include_prereleases&label=download&color=8ab4f8" alt="Download"></a>
  <img src="https://img.shields.io/badge/status-beta-orange" alt="Beta">
  <img src="https://img.shields.io/badge/TruckersMP-compatible-brightgreen" alt="TruckersMP compatible">
  <img src="https://img.shields.io/badge/platform-Windows-blue" alt="Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-lightgrey" alt="GPL-3.0"></a>
</p>

<p align="center"><img src="docs/images/home.png" width="820" alt="TruckersTV's home screen"></p>

> **Beta.** Expect rough edges, and for now the TV only fits the Iveco S-Way. Bugs and ideas are welcome in [Issues](https://github.com/DrLemony/TruckersTV/issues).

## Download

Get the installer from the [latest release](https://github.com/DrLemony/TruckersTV/releases) and run it. It isn't code signed yet, so Windows SmartScreen may warn you: choose *More info*, then *Run anyway*.

## What it is

TruckersTV is a Windows app and a game plugin. The TV is bought in the game's own truck shop, under *Bed toys*, as "TruckersTV TV" (1500 €) or the bigger "TruckersTV TV XL" (3000 €) and "TruckersTV TV XXL" (4500 €). It is drawn into the cab, and the truck's own screens stay the game's. There's no mod to install by hand, so it also runs under TruckersMP.

For now the TV is placed in the Iveco S-Way, over the right door.

## Using it

1. Open TruckersTV before the game and go through its setup.
2. In the game, buy the TV for your truck.
3. Hold the interaction key to take the mouse: the cursor moves on the TV, a click clicks, the keyboard types.

## Screens

| | |
|---|---|
| ![Apps](docs/images/apps.png) | ![Search with the on-screen keyboard](docs/images/search.png) |
| ![Spotify](docs/images/spotify.png) | ![YouTube Music](docs/images/youtube-music.png) |

The companion app on the PC shows the TV's state, its keys and your accounts:

<p align="center"><img src="docs/images/companion.png" width="600" alt="The companion app"></p>

*Taken from the app's preview, outside the game: the artists and songs shown are sample data.*

## Türkçe

TruckersTV, Euro Truck Simulator 2'de tırın kabinine televizyon koyar: YouTube, müzik, canlı TV, tarayıcı ve kendi videoların, Android TV tarzı bir ana ekranla. Şu an beta ve yalnızca Iveco S-Way'de çalışıyor; TruckersMP ile uyumlu.

[Son sürümden](https://github.com/DrLemony/TruckersTV/releases) kurulum dosyasını indir ve çalıştır, sonra oyundan önce TruckersTV'yi aç. Televizyonu oyunda tır galerisinden, *Yatak Eşyaları* altından satın alırsın. Windows uyarı verirse *Ek bilgi*, ardından *Yine de çalıştır*'a bas.

## Building

```bash
npm ci --prefix ui
npm run build:release --prefix ui
cargo build --release -p westplay-app -p westplay-plugin --features tauri/custom-protocol
```

## Credits and license

TruckersTV is built on [WestPlay](https://github.com/enes-less/WestPlay) by enes-less, and its browser core comes from [Helow19274/WestPlay](https://github.com/Helow19274/WestPlay). Like them it's licensed under the GPL-3.0-or-later: see [LICENSE](LICENSE). The crates keep their `westplay-` names.

Euro Truck Simulator 2 is a trademark of SCS Software. TruckersTV isn't affiliated with SCS Software, TruckersMP, Google or the services it opens. Android TV is a trademark of Google, named only to describe the look.
