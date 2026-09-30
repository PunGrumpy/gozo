import Link from "next/link";

import { CopyButton } from "@/components/copy-button";
import { Kit } from "@/components/kit";
import {
  Block,
  Feature,
  ScrollCue,
  SiteFooter,
  SiteHeader,
} from "@/components/site";
import { Dim, Line, Prompt, Term, True, Url } from "@/components/term";
import { Scene } from "@/components/ui";
import { button } from "@/lib/button";

const INSTALL = "npm i -g gozo";

const stats = [
  { label: "commands, one binary", value: "18" },
  { label: "platforms, one npm package", value: "3" },
  { label: "runtime dependencies", value: "0" },
];

/* where each word of the statement starts before it settles, cycling every four words */
const scatter = [
  [-40, 18],
  [28, -14],
  [72, 10],
  [-16, -24],
];

const words = (text: string, offset = 0) =>
  text.split(" ").map((word, i) => {
    const [dx, dy] = scatter[(offset + i) % scatter.length];
    return (
      <span
        className="scroll-motion:settle inline-block whitespace-pre"
        key={word}
        style={{ "--dx": `${dx}px`, "--dy": `${dy}px` }}
      >
        {word}{" "}
      </span>
    );
  });

const enter =
  "transition-[opacity,translate] duration-600 ease-out starting:translate-y-3 starting:opacity-0 motion-reduce:transition-none";

const HomePage = () => (
  <div className="relative mx-auto max-w-[1280px] px-5 md:px-10">
    <SiteHeader />

    <section className="md:min-h-[190svh]">
      <div className="relative flex min-h-[88svh] flex-col justify-end pb-26 md:sticky md:top-0 md:min-h-svh">
        <Kit />
        <h1
          className={`text-display -ml-[0.04em] max-w-[14ch] font-serif font-normal ${enter}`}
        >
          the joy of go, handled.
        </h1>
        <p
          className={`mt-7 max-w-[448px] text-[17px] text-gray-900 delay-80 ${enter}`}
        >
          gozo is one binary that gives go projects the workflow of vercel cli.
          run it in any repo and it knows the module, the workspace, the
          toolchain and where you deploy.
        </p>
        <div className={`mt-7 flex flex-wrap gap-2.5 delay-160 ${enter}`}>
          <a className={button()} href="#install">
            install gozo
          </a>
          <Link className={button({ ghost: true })} href="/docs">
            read the docs
          </Link>
        </div>
        <ScrollCue />
      </div>
    </section>

    <Block glyph="▲" x="c">
      <Term>
        <Prompt>gozo deploy --prod</Prompt>
        <Line mark="step">
          deploying api to kubernetes <Dim>(production)</Dim>
        </Line>
        <Line mark="ok">docker 27.1.0</Line>
        <Line mark="ok">
          deployed ghcr.io/acme/api:3f9c2a1 <Dim>[42s]</Dim>
        </Line>
        <Line mark="warn">2 dependencies can be updated</Line>
        <Url>https://api.acme.dev</Url>
      </Term>
    </Block>

    <Block from="c" glyph="◆" x="l">
      <blockquote className="max-w-[640px] pl-(--gutter)">
        <p className="font-serif text-[clamp(32px,4vw,52px)] leading-[1.1] tracking-[-0.02em] [hanging-punctuation:first]">
          stdout is the result. stderr is the story.
        </p>
        <footer className="mt-5 text-[13px] text-gray-900">
          the rule every gozo command follows
        </footer>
      </blockquote>
    </Block>

    <Feature
      more={{ href: "/docs/commands#gozo-dev", label: "gozo dev" }}
      points={[
        "env files layered: .env, .env.local, then .env.production.",
        "compose services started before your app is.",
        "rebuild and restart on every save, with the build error inline.",
      ]}
      title="dev that knows your repo."
      x="l"
    >
      <Term>
        <Prompt>gozo dev</Prompt>
        <Line mark="step">detected ./cmd/api</Line>
        <Line mark="step">
          loaded .env, .env.local <Dim>(6 vars)</Dim>
        </Line>
        <Line mark="step">
          <b>ready!</b> <Url>http://localhost:8080</Url> <Dim>[1.2s]</Dim>
        </Line>
        <Line mark="step">
          handler.go changed, rebuilt <Dim>[0.4s]</Dim>
        </Line>
      </Term>
    </Feature>

    <Feature
      from="l"
      glyph="●"
      more={{ href: "/docs/commands#gozo-doctor", label: "gozo doctor" }}
      points={[
        "toolchain, go.mod, go.work, tidy and cgo checked together.",
        "check, test and build run across every module in go.work.",
        "outdated dependencies and stray replace directives called out.",
      ]}
      title="one report for the whole workspace."
      x="c"
    >
      <Term>
        <Prompt>gozo doctor</Prompt>
        <Line mark="ok">go toolchain 1.25.1</Line>
        <Line mark="ok">go 1.25.1 satisfies go directive 1.24</Line>
        <Line mark="ok">go.work covers 3 modules</Line>
        <Line mark="warn">2 dependencies can be updated</Line>
        <Line mark="fail">
          cgo enabled but c compiler <Dim>gcc</Dim> is not on path
        </Line>
      </Term>
    </Feature>

    <Block as="div" className="py-10 pl-(--gutter)" from="c" glyph="◆" x="l">
      <Scene
        alt="a blurred figure waiting at a crossing under a teal evening sky"
        body="waiting on the light with nothing left to carry."
        className="md:aspect-[16/7]"
        file="crossing"
        name="the crossing"
      />
    </Block>

    <Feature
      more={{ href: "/docs/deploy", label: "deploy targets" }}
      points={[
        "link a directory to docker or kubernetes once.",
        "deploy, watch logs, roll back to the last good one.",
        "every deployment recorded, so rollback is one command.",
      ]}
      title="deploy, then go home."
      x="l"
    >
      <Term>
        <Prompt>gozo rollback</Prompt>
        <Line mark="step">
          rolling api back to 8c1d0f4 <Dim>(2h ago)</Dim>
        </Line>
        <Line mark="ok">
          rollout complete <Dim>[9s]</Dim>
        </Line>
        <Prompt>gozo status</Prompt>
        <Line mark="ok">
          api healthy, 3/3 ready <Dim>since 9s</Dim>
        </Line>
      </Term>
    </Feature>

    <Feature
      from="l"
      glyph="●"
      more={{ href: "/docs/agents", label: "for agents" }}
      points={[
        "every command answers in json with a schema id.",
        "exit codes mean the same thing everywhere: 0, 1 or 2.",
        "ci and coding agents are detected, so nothing waits on a prompt.",
      ]}
      title="built for agents too."
      x="c"
    >
      <Term>
        <Prompt>gozo status --json</Prompt>
        {"{\n"}
        {'  "schema": '}
        <Url>&quot;gozo.status/v1&quot;</Url>
        {",\n"}
        {'  "project": "api",\n'}
        {'  "target": "kubernetes",\n'}
        {'  "healthy": '}
        <True />
        {",\n"}
        {'  "ready": 3,\n'}
        {'  "desired": 3\n'}
        {"}"}
      </Term>
    </Feature>

    <Block from="c" glyph="◆" x="l">
      <div className="ml-(--gutter) grid gap-8 md:grid-cols-3">
        {stats.map((s) => (
          <div className="border-t border-gray-400 pt-5" key={s.label}>
            <b className="block font-serif text-[clamp(56px,6vw,88px)] leading-none font-normal tracking-[-0.03em] tabular-nums">
              {s.value}
            </b>
            <span className="mt-2.5 block font-mono text-xs tracking-[0.06em] text-gray-900">
              {s.label}
            </span>
          </div>
        ))}
      </div>
    </Block>

    <Block as="div" className="py-10 pl-(--gutter)" x="l">
      <Scene
        alt="two blurred figures walking into warm last light"
        body="that's how things ship before dinner."
        className="md:aspect-[16/7]"
        file="last-light"
        name="last light"
      />
    </Block>

    <Block className="[view-timeline-name:--statement]" reveal={false} x="l">
      <p className="max-w-[26ch] pl-(--gutter) text-[clamp(28px,3.4vw,43px)] leading-[1.05] tracking-[-0.01em]">
        <span className="block">
          {words("you don't need a platform team.")}
        </span>
        <span className="block">{words("you don't need a makefile.")}</span>
        <span className="block">
          {words("just")}
          <span
            className="scroll-motion:settle inline-block font-serif whitespace-pre italic"
            style={{ "--dx": "28px", "--dy": "-14px" }}
          >
            gozo deploy{" "}
          </span>
          {words("and go home.", 2)}
        </span>
      </p>
    </Block>

    <Block id="install" x="l">
      <div className="max-w-[900px] pl-(--gutter)">
        <div className="border-gray-1000 flex items-center justify-between gap-6 border-b pt-1.5 pb-3 font-mono text-[clamp(24px,3.4vw,43px)] normal-case">
          <span className="min-w-0 wrap-anywhere">{INSTALL}</span>
          <CopyButton text={INSTALL} />
        </div>
        <p className="mt-4.5 text-[13px] wrap-anywhere text-gray-900">
          or <code className="mx-1 normal-case">bun add -g gozo</code> ·{" "}
          <code className="mx-1 normal-case">cargo install gozo</code> ·{" "}
          <code className="mx-1 normal-case">
            curl -fsSL
            https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh |
            sh
          </code>
        </p>
      </div>
    </Block>

    <SiteFooter from="l" />
  </div>
);

export default HomePage;
