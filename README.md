# kemono

Downloader for [kemono.su](https://kemono.su)

## Installation

```bash
cargo install --git https://github.com/kawaii-hentai-dev/kemono/ --bin kemono-cli
```

## Usage

```text
$ kemono-cli --help
Usage: kemono-cli [OPTIONS] <URL>

Arguments:
  <URL>
          kemono user URL to fetch posts
          
          Example: https://kemono.su/fanbox/user/4107959

Options:
      --output-dir <OUTPUT_DIR>
          Output directory of fetched posts
          
          [default: ./download]

  -p, --max-concurrency <MAX_CONCURRENCY>
          Maximium number of tasks running in background concurrently
          
          [default: 4]

  -w, --whitelist-regex <WHITELIST_REGEX>
          Whitelist regex for title
          
          Specify multiple times means 'AND' semantic

  -b, --blacklist-regex <BLACKLIST_REGEX>
          Blacklist regex for title
          
          Specify multiple times means 'AND' semantic

  -W, --whitelist-filename-regex <WHITELIST_FILENAME_REGEX>
          Whitelist regex for filename
          
          Specify multiple times means 'AND' semantic

  -B, --blacklist-filename-regex <BLACKLIST_FILENAME_REGEX>
          Blacklist regex for filename
          
          Specify multiple times means 'AND' semantic

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Example

1. Download 4k videos, with title starts with `Melody x Lawa`

```bash
kemono-cli https://kemono.su/patreon/user/49965584 -w "Melody x Lawa" -W "\.mp4$" -W "4K"
```

```text
download/ViciNeko
├── Melody x Lawa - A Taste Of The Exotic (¬‿¬)❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene24_ATasteOfTheExotic_4K_h265.mp4
├── Melody x Lawa - At the Helm ❤ (⁄ ⁄ ⁄ω⁄ ⁄ ⁄)⁄ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene22_AtTheHelm_4K_h265.mp4
├── Melody x Lawa - Challenge Accepted ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene23-1_ChallengeAccepted_4K_h265.mp4
├── Melody x Lawa - Containment Breach ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene5_ContainmentBreach_4K_h265.mp4
├── Melody x Lawa - Let's Get Started! ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene12_LetsGetStarted_4K_h265.mp4
├── Melody x Lawa - Next Challenge: Paizuri! ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP + A Nut Between Worlds Teaser
│   ├── metadata.json
│   └── MxL_WIP_Scene27mk2_NextChallengePaizuri!_4K_h265.mp4
├── Melody x Lawa - Nice Robot... ❤ (⁄ ⁄ ⁄ω⁄ ⁄ ⁄)⁄ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene21-3_NiceRobot_4K_h265.mp4
├── Melody x Lawa - Overflow (¬‿¬)❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene32mk2_Overflow_4K_h265.mp4
├── Melody x Lawa - Rise of the Machine ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene8_RiseOfThe Machine_4K_h265.mp4
├── Melody x Lawa - Rise to the Challenge ❤ ٩(๑•̀ω•́๑)۶ ❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene21-2_RiseToTheChallenge_4K_h265.mp4
├── Melody x Lawa - Robot Rodeo (¬‿¬)❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene34_RobotRodeo_4K_h265.mp4
├── Melody x Lawa - Technical Difficulties (¬‿¬)❤ - WIP
│   ├── metadata.json
│   └── MxL_Scene17_TechnicalDifficulties_4K_h265.mp4
├── Melody x Lawa - Tongue Twister ❤(˵> ◡ <˵)❤ - WIP
│   ├── metadata.json
│   └── MxL_WIP_Scene28mk2_TongueTwister_4K_h265.mp4
└── Melody x Lawa - Welcome To The Show! ❤(˵^ ◡ ^˵)❤ - WIP
    ├── metadata.json
    └── MxL_WIP_Scene1_WelcomeToTheShow_4K_h265.mp4
```

2. Download videos of HongkongDoll from coomer.su

```bash
kemono-cli https://coomer.su/onlyfans/user/hongkongdoll -W "\.(mp4|m4v)$" --coomer
```