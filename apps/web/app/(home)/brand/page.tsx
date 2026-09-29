import type { Metadata } from "next";
import Link from "next/link";

import { REPO, SiteFooter, SiteHeader, Wire } from "@/components/site";

export const metadata: Metadata = {
  description: "gozo logos, type, colour and scenes. grab what you need.",
  title: "brand",
};

const lockups = [
  { file: "logo-ink.svg", name: "lockup", tone: "paper" },
  { file: "logo-paper.svg", name: "lockup", tone: "ink" },
  { file: "symbol-ink.svg", name: "symbol", tone: "paper" },
  { file: "symbol-paper.svg", name: "symbol", tone: "ink" },
] as const;

const colours = [
  { hex: "#101828", name: "ink", use: "text, wordmark, buttons" },
  { hex: "#fcfcfd", name: "paper", use: "backgrounds" },
  { hex: "#667085", name: "ash", use: "secondary text" },
  { hex: "#e4e7ec", name: "line", use: "borders, dividers" },
  { hex: "#0e1416", name: "deep", use: "scenes, shadow" },
  { hex: "#15535f", name: "teal", use: "scenes, sky" },
  { hex: "#6992a5", name: "mist", use: "scenes, haze" },
  { hex: "#f2b26a", name: "tungsten", use: "scenes, light" },
  { hex: "#e0432a", name: "signal", use: "scenes, one accent" },
];

const voice = [
  {
    not: "🚀 successfully deployed your application!",
    say: "deployed. logs are one command away.",
  },
  {
    not: "supercharge your developer productivity",
    say: "ship at five. dinner at six.",
  },
  {
    not: "powerful dependency management built in",
    say: "it never forgets to tidy, so you don't either.",
  },
  {
    not: "the all-in-one toolkit for modern go development",
    say: "one binary. every go workflow.",
  },
];

const scenes = [
  {
    alt: "a blurred figure waiting at a crossing under a teal evening sky",
    body: "waiting on the light with nothing left to carry.",
    file: "crossing",
    name: "the crossing",
  },
  {
    alt: "two blurred figures walking into warm last light",
    body: "the floodlight comes on before the sky goes. somebody already went home.",
    file: "last-light",
    name: "last light",
  },
  {
    alt: "rain on glass, a figure under mist",
    body: "it rained anyway. the deploy did not notice.",
    file: "weather",
    name: "weather",
  },
  {
    alt: "a table after dinner, tungsten light, nobody looking at a phone",
    body: "dinner ran long. nobody checked a phone.",
    file: "after-hours",
    name: "after hours",
  },
];

const BrandPage = () => (
  <div className="page">
    <SiteHeader />

    <section className="hero short">
      <h1>brand.</h1>
      <p className="lede">
        logos, type, colour and a few scenes. grab what you need, do not alter
        the files.
      </p>
      <div className="ctas">
        <a className="btn" href={`${REPO}/tree/main/brand`}>
          download all assets
        </a>
        <Link className="btn ghost" href="/">
          back home
        </Link>
      </div>
      <div aria-hidden="true" className="scroll">
        <span>s</span>
        <span>c</span>
        <span>r</span>
        <span>o</span>
        <span>l</span>
        <span>l</span>
      </div>
    </section>

    <section className="feature" data-reveal data-x="c">
      <Wire glyph="▲" />
      <div className="copy">
        <h2>gozo. lowercase, always.</h2>
        <ul className="points">
          <li>one word, even at the start of a sentence.</li>
          <li>never gozo cli, gozo.dev, Gozo or GoZo as the name.</li>
          <li>in spanish and portuguese gozo means joy. that is the point.</li>
        </ul>
      </div>
      <div className="card paper stage">
        <div aria-hidden="true" className="wordmark" />
        <span className="sr-only">gozo</span>
      </div>
    </section>

    <section data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <h2 className="title">wordmark for recognition, symbol when tight.</h2>
      <div className="assets">
        {lockups.map((l) => (
          <div className={`card ${l.tone}`} key={l.file}>
            <div className="stage">
              <img
                alt={`gozo ${l.name}`}
                height={72}
                src={`/brand/${l.file}`}
              />
            </div>
            <div className="cap">
              <span>
                <b>{l.name}</b> · {l.tone === "paper" ? "ink" : "paper"}
              </span>
              <a className="more" download href={`/brand/${l.file}`}>
                svg
              </a>
            </div>
          </div>
        ))}
        <div className="card paper">
          <div className="stage">
            <img alt="gozo favicon" height={64} src="/brand/favicon.svg" />
          </div>
          <div className="cap">
            <span>
              <b>favicon</b> · 64, 32, 16
            </span>
            <a className="more" download href="/brand/favicon.svg">
              svg
            </a>
          </div>
        </div>
        <div className="card paper">
          <div className="stage">
            <img
              alt="gozo social preview card"
              className="preview"
              src="/brand/social-preview.png"
            />
          </div>
          <div className="cap">
            <span>
              <b>social preview</b> · 1280 × 640
            </span>
            <a className="more" download href="/brand/social-preview.png">
              png
            </a>
          </div>
        </div>
      </div>
    </section>

    <section className="feature" data-reveal data-x="c">
      <Wire from="l" glyph="●" />
      <div className="copy">
        <h2>three faces, three jobs.</h2>
        <ul className="points">
          <li>instrument serif for display. regular only, tracking −2%.</li>
          <li>geist for reading and ui. 400, 500 and 600.</li>
          <li>geist mono for the terminal. tabular, ligatures off.</li>
        </ul>
      </div>
      <div className="card spec">
        <span className="d">the joy of go, handled.</span>
        <span className="b">
          gozo is one binary that gives go projects the workflow of vercel cli.
          run it in any repo and it knows the module, the workspace, the
          toolchain and where you deploy.
        </span>
        <span className="m">
          <span className="dim">$</span> gozo deploy --prod{" "}
          <span className="dim">[42s]</span>
        </span>
      </div>
    </section>

    <section data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <h2 className="title">ink and paper. the scenes bring the rest.</h2>
      <div className="swatches">
        {colours.map((c) => (
          <div className="card" key={c.name}>
            <div className={`chip chip-${c.name}`} />
            <div className="cap">
              <b>{c.name}</b>
              <span>
                {c.hex} · {c.use}
              </span>
            </div>
          </div>
        ))}
      </div>
    </section>

    <section className="feature" data-reveal data-x="c">
      <Wire from="l" glyph="●" />
      <div className="copy">
        <h2>say what happened, then stop.</h2>
        <ul className="points">
          <li>short sentences that end. second person.</li>
          <li>calm, never hyped. no emoji, no exclamation marks.</li>
          <li>lowercase in the product, sentence case in the docs.</li>
        </ul>
      </div>
      <ul className="card pairs">
        {voice.map((v) => (
          <li key={v.say}>
            <span className="say">{v.say}</span>
            <span className="not">{v.not}</span>
          </li>
        ))}
      </ul>
    </section>

    <section data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <h2 className="title">scenes.</h2>
      <div className="scenes">
        {scenes.map((s) => (
          <figure className="scene" key={s.file}>
            <img alt={s.alt} src={`/brand/scenes/${s.file}.jpg`} />
            <figcaption>
              <b>{s.name}</b>
              <span>{s.body}</span>
            </figcaption>
          </figure>
        ))}
      </div>
    </section>

    <SiteFooter from="l" />
  </div>
);

export default BrandPage;
