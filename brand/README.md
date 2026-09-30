# gozo brand

Open `index.html` for the brand page. This file is the written version.

## Naming

`gozo` is always lowercase, even at the start of a sentence, and always one word. Never Gozo, GoZo, gozo CLI or gozo.dev as the name. In Spanish and Portuguese, _gozo_ means joy. That is the point of the tool.

## Positioning

- Tagline: The joy of Go.
- Descriptor: The missing developer experience layer for Go.
- Support: One binary. Every Go workflow.

## Logo

Wordmark for recognition, symbol when space is tight, lockup when both fit. Everything ships in ink and paper.

The symbol is the seed: three petals folded together, about to open. gozo means joy, and the seed is the version of it that fits in a favicon. Construction: one leaf shape (length 44, width 19 on a 96 unit grid) repeated at 120°, the whole mark tilted 15° so it leans forward, with a 3 unit gap at the centre. The gap is what keeps it readable at 12 px.

The wordmark is `gozo` in Instrument Serif with 2% negative tracking, outlined to paths. In the lockup the symbol sits left of the wordmark at 90% of its cap height, with a gap of half the symbol width.

- `symbol-ink.svg`, `symbol-paper.svg`
- `wordmark-ink.svg`, `wordmark-paper.svg`
- `logo-ink.svg`, `logo-paper.svg`
- `favicon.svg` (ink rounded square, paper seed), `social-preview.svg` and `.png` (1280 × 640)

Clear space is one petal length on every side. Minimum size is 12 px for the symbol and 20 px tall for the wordmark. Do not rotate the seed further, recolour it, or add more petals.

## Typography

| Role | Face | Notes |
| --- | --- | --- |
| Headlines and display | Instrument Serif 400 | 88 / 44 / 28 px, tracking −2.5% to −1.5%, line height 0.95 to 1.05 |
| Body and interface | Geist 400 to 600 | 16 / 24, 19 px for ledes, 13 to 15 px for controls |
| Code and the terminal | Geist Mono 400 | 14 / 24 |

All three are open source (SIL OFL). Instrument Serif is on Google Fonts; Geist and Geist Mono are published by Vercel and also on Google Fonts.

## Colour

The interface uses Geist's token names and scale, defined in `tokens.css`. Steps 100 to 300 are component backgrounds, 400 to 600 borders, 700 and 800 high-contrast backgrounds, 900 and 1000 text. Every token has a light and a dark value.

| Token            | Light     | Use                   |
| ---------------- | --------- | --------------------- |
| `gray-1000`      | `#11151C` | text, primary buttons |
| `background-100` | `#FCFCFD` | page background       |
| `background-200` | `#F6F6F8` | cards                 |
| `gray-900`       | `#3A455F` | secondary text        |
| `gray-400`       | `#E8E9ED` | borders and dividers  |

The logo files keep their own ink `#101828` and paper `#FCFCFD`, so they read the same outside the site.

There is no accent colour. Colour belongs to the terminal: green for `✓`, yellow for `!`, red for `✗`, dim for secondary lines, cyan for links. gozo uses the ANSI palette so it matches whatever theme the user already has.

## Voice

Short sentences that end. Second person. Calm, never hyped. Say what happened, then stop.

| Say | Not |
| --- | --- |
| Deployed. Logs are one command away. | 🚀 Successfully deployed your application! |
| Ship at five. Dinner at six. | Supercharge your developer productivity |
| It never forgets to tidy, so you don’t either. | Powerful dependency management built in |
| One binary. Every Go workflow. | The all-in-one toolkit for modern Go development |

## Scenes

Photography follows the Orchid brand backgrounds: real photographs of people, blurred until they become shapes, with heavy film grain and a poetic caption. What is ours is the grade. Every frame is blue hour in a teal world, and the single warm light in it (a floodlight, a raincoat, a sign, a lamp on a table) is the subject. Nobody in the picture is looking at a screen; the work is already done.

Palette, pulled from four colour references (street photographs at dusk in Bangkok and Tokyo):

| Name     | Hex       | Where it comes from               |
| -------- | --------- | --------------------------------- |
| Deep     | `#0e1416` | shadows, the base of every scene  |
| Teal     | `#15535f` | dusk sky, wet asphalt             |
| Mist     | `#6992a5` | haze and reflections              |
| Tungsten | `#f2b26a` | the single warm light             |
| Signal   | `#e0432a` | the one red thing, used sparingly |

Rules: one warm light per frame, never two. Motion blur or shallow focus until faces disappear. Portrait 4:5 for cards, 3:2 for headers and social. No lens flare, no HDR, no grading past teal and tungsten. Grain lives in the image file, never on the page.

The four scenes, each with a title and one sentence:

- The crossing: waiting on the light with nothing left to carry. It shipped before dinner. (`scenes/crossing.jpg`, header)
- Last light: the floodlight comes on before the sky goes. Somebody already went home. (`scenes/last-light.jpg`)
- Weather: it rained anyway. The deploy did not notice. (`scenes/weather.jpg`)
- After hours: dinner ran long. Nobody checked a phone. (`scenes/after-hours.jpg`)

The files in `scenes/` are generated placeholders: gradients in the palette with film grain baked into the pixels at about 30%, so the texture survives resizing and compression. They hold the slots until real photographs that follow the rules above exist. The grain in the CSS overlay is a light 14% on top; the image itself must already carry the grain.

## Layout

Page max width 1440 px with 120 px gutters, prose max width 620 px. Corners 16 px on cards, 22 px on hero images, pill for buttons. One pixel `Line` borders, no shadows.
