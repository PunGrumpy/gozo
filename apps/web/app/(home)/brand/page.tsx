import type { Metadata } from "next";
import Link from "next/link";
import type { ReactNode } from "react";

import {
  Block,
  Feature,
  REPO,
  ScrollCue,
  SiteFooter,
  SiteHeader,
} from "@/components/site";
import { More, Scene, Title } from "@/components/ui";
import { button } from "@/lib/button";
import { cn } from "@/lib/cn";

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

/* the interface palette is the Geist tokens (light values shown); the scene palette is for photography only */
const tokens = [
  {
    chip: "bg-gray-1000",
    hex: "#11151c",
    name: "gray-1000",
    use: "text, buttons",
  },
  {
    chip: "bg-background-100",
    hex: "#fcfcfd",
    name: "background-100",
    use: "page",
  },
  {
    chip: "bg-background-200",
    hex: "#f6f6f8",
    name: "background-200",
    use: "cards",
  },
  {
    chip: "bg-gray-900",
    hex: "#3a455f",
    name: "gray-900",
    use: "secondary text",
  },
  { chip: "bg-gray-400", hex: "#e8e9ed", name: "gray-400", use: "borders" },
];

const sceneColours = [
  { hex: "#0e1416", name: "deep", use: "shadow" },
  { hex: "#15535f", name: "sea", use: "sky" },
  { hex: "#6992a5", name: "mist", use: "haze" },
  { hex: "#f2b26a", name: "tungsten", use: "light" },
  { hex: "#e0432a", name: "signal", use: "one accent" },
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

const card = "rounded-2xl border border-gray-400 bg-background-200 p-6";

/* lockup cards show the files on their own background, whatever the page theme */
const tone = {
  ink: "light border-gray-1000 bg-gray-1000 text-background-100",
  paper: "light bg-background-100 text-gray-1000",
};

const Asset = ({
  alt,
  file,
  format,
  height,
  label,
  preview = false,
  toneName,
}: {
  alt: string;
  file: string;
  format: string;
  height?: number;
  label: ReactNode;
  preview?: boolean;
  toneName: keyof typeof tone;
}) => (
  <div className={cn(card, tone[toneName], "flex flex-col gap-4")}>
    <div className="grid min-h-40 flex-1 place-items-center">
      <img
        alt={alt}
        className={cn(
          "block max-w-full",
          preview &&
            "outline-gray-alpha-300 h-auto w-full rounded-lg outline-1 -outline-offset-1"
        )}
        height={height}
        src={`/brand/${file}`}
      />
    </div>
    <div className="flex items-baseline justify-between gap-3 font-mono text-[13px]">
      <span>{label}</span>
      <More download href={`/brand/${file}`}>
        {format}
      </More>
    </div>
  </div>
);

const Swatch = ({
  chip,
  hex,
  name,
  use,
}: {
  chip: string;
  hex: string;
  name: string;
  use: string;
}) => (
  <div className={cn(card, "p-2")}>
    <div className="light">
      <div
        className={cn(
          "outline-gray-alpha-300 h-30 rounded-lg outline-1 -outline-offset-1",
          chip
        )}
        style={chip ? undefined : { background: hex }}
      />
    </div>
    <div className="flex flex-col gap-0.5 px-2 pt-3 pb-1.5 font-mono text-[13px]">
      <b className="font-medium">{name}</b>
      <span className="text-[11px] text-gray-900">
        {hex} · {use}
      </span>
    </div>
  </div>
);

const grid = "ml-(--gutter) grid gap-4";

const BrandPage = () => (
  <div className="relative mx-auto max-w-[1280px] px-5 md:px-10">
    <SiteHeader />

    <section className="relative pt-30 pb-32">
      <h1 className="text-display -ml-[0.04em] font-serif font-normal">
        brand.
      </h1>
      <p className="mt-7 max-w-[448px] text-[17px] text-gray-900">
        logos, type, colour and a few scenes. grab what you need, do not alter
        the files.
      </p>
      <div className="mt-7 flex flex-wrap gap-2.5">
        <a className={button()} href={`${REPO}/tree/main/brand`}>
          download all assets
        </a>
        <Link className={button({ ghost: true })} href="/">
          back home
        </Link>
      </div>
      <ScrollCue />
    </section>

    <Feature
      glyph="▲"
      points={[
        "one word, even at the start of a sentence.",
        "never gozo cli, gozo.dev, Gozo or GoZo as the name.",
        "in spanish and portuguese gozo means joy. that is the point.",
      ]}
      title="gozo. lowercase, always."
      x="c"
    >
      <div className={cn(card, tone.paper, "grid min-h-70 place-items-center")}>
        <div
          aria-hidden="true"
          className="aspect-[242/160] w-3/5 bg-current mask-[url(/brand/wordmark-ink.svg)] mask-contain mask-center mask-no-repeat"
        />
        <span className="sr-only">gozo</span>
      </div>
    </Feature>

    <Block from="c" glyph="◆" x="l">
      <Title className="mb-8 ml-(--gutter)">
        wordmark for recognition, symbol when tight.
      </Title>
      <div className={cn(grid, "md:grid-cols-2")}>
        {lockups.map((l) => (
          <Asset
            alt={`gozo ${l.name}`}
            file={l.file}
            format="svg"
            height={72}
            key={l.file}
            label={
              <>
                <b className="font-medium">{l.name}</b> ·{" "}
                {l.tone === "paper" ? "ink" : "paper"}
              </>
            }
            toneName={l.tone}
          />
        ))}
        <Asset
          alt="gozo favicon"
          file="favicon.svg"
          format="svg"
          height={64}
          label={
            <>
              <b className="font-medium">favicon</b> · 64, 32, 16
            </>
          }
          toneName="paper"
        />
        <Asset
          alt="gozo social preview card"
          file="social-preview.png"
          format="png"
          label={
            <>
              <b className="font-medium">social preview</b> · 1280 × 640
            </>
          }
          preview
          toneName="paper"
        />
      </div>
    </Block>

    <Feature
      from="l"
      glyph="●"
      points={[
        "instrument serif for display. regular only, tracking −2%.",
        "geist for reading and ui. 400, 500 and 600.",
        "geist mono for the terminal. tabular, ligatures off.",
      ]}
      title="three faces, three jobs."
      x="c"
    >
      <div className={cn(card, "grid gap-6 p-8")}>
        <span className="font-serif text-[clamp(40px,4vw,56px)] leading-none tracking-[-0.025em]">
          the joy of go, handled.
        </span>
        <span>
          gozo is one binary that gives go projects the workflow of vercel cli.
          run it in any repo and it knows the module, the workspace, the
          toolchain and where you deploy.
        </span>
        <span className="font-mono text-sm normal-case">
          <span className="text-gray-900">$</span> gozo deploy --prod{" "}
          <span className="text-gray-900">[42s]</span>
        </span>
      </div>
    </Feature>

    <Block from="c" glyph="◆" x="l">
      <Title className="mb-8 ml-(--gutter)">
        geist tokens for the interface. the scenes bring the rest.
      </Title>
      <div
        className={cn(grid, "grid-cols-[repeat(auto-fill,minmax(180px,1fr))]")}
      >
        {tokens.map((t) => (
          <Swatch key={t.name} {...t} />
        ))}
        {sceneColours.map((c) => (
          <Swatch chip="" key={c.name} {...c} />
        ))}
      </div>
    </Block>

    <Feature
      from="l"
      glyph="●"
      points={[
        "short sentences that end. second person.",
        "calm, never hyped. no emoji, no exclamation marks.",
        "lowercase in the product, sentence case in the docs.",
      ]}
      title="say what happened, then stop."
      x="c"
    >
      <ul className={cn(card, "px-6 py-2")}>
        {voice.map((v) => (
          <li
            className="border-b border-gray-400 py-3.5 last:border-b-0"
            key={v.say}
          >
            <span className="block text-[17px]">{v.say}</span>
            <span className="mt-1 block text-[13px] text-gray-900 line-through">
              {v.not}
            </span>
          </li>
        ))}
      </ul>
    </Feature>

    <Block from="c" glyph="◆" x="l">
      <Title className="mb-8 ml-(--gutter)">scenes.</Title>
      <div className={cn(grid, "md:grid-cols-2")}>
        {scenes.map((s) => (
          <Scene
            alt={s.alt}
            body={s.body}
            className="md:aspect-[16/10]"
            file={s.file}
            key={s.file}
            name={s.name}
          />
        ))}
      </div>
    </Block>

    <SiteFooter from="l" />
  </div>
);

export default BrandPage;
