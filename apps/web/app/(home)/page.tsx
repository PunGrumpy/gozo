import Link from "next/link";

import { CopyButton } from "@/components/copy-button";
import { Kit } from "@/components/kit";
import { SiteFooter, SiteHeader, Wire } from "@/components/site";
import { Line, Prompt, Term } from "@/components/term";

const INSTALL = "npm i -g gozo";

const stats = [
  { label: "commands, one binary", value: "18" },
  { label: "platforms, one npm package", value: "3" },
  { label: "runtime dependencies", value: "0" },
];

const words = (text: string) =>
  text.split(" ").map((word) => (
    <span className="w" key={word}>
      {word}{" "}
    </span>
  ));

const HomePage = () => (
  <div className="page">
    <SiteHeader />

    <section className="hero">
      <div className="stage">
        <Kit />
        <h1>the joy of go, handled.</h1>
        <p className="lede">
          gozo is one binary that gives go projects the workflow of vercel cli.
          run it in any repo and it knows the module, the workspace, the
          toolchain and where you deploy.
        </p>
        <div className="ctas">
          <a className="btn" href="#install">
            install gozo
          </a>
          <Link className="btn ghost" href="/docs">
            read the docs
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
      </div>
    </section>

    <section data-reveal data-x="c">
      <Wire glyph="▲" />
      <Term>
        <Prompt>gozo deploy --prod</Prompt>
        <Line mark="step">
          deploying api to kubernetes <span className="dim">(production)</span>
        </Line>
        <Line mark="ok">docker 27.1.0</Line>
        <Line mark="ok">
          deployed ghcr.io/acme/api:3f9c2a1 <span className="dim">[42s]</span>
        </Line>
        <Line mark="warn">2 dependencies can be updated</Line>
        <span className="link">https://api.acme.dev</span>
      </Term>
    </section>

    <section data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <blockquote className="quote">
        <p>stdout is the result. stderr is the story.</p>
        <footer>the rule every gozo command follows</footer>
      </blockquote>
    </section>

    <section className="feature" data-reveal data-x="l">
      <Wire />
      <div className="copy">
        <h2>dev that knows your repo.</h2>
        <ul className="points">
          <li>env files layered: .env, .env.local, then .env.production.</li>
          <li>compose services started before your app is.</li>
          <li>
            rebuild and restart on every save, with the build error inline.
          </li>
        </ul>
        <Link className="more" href="/docs/commands#gozo-dev">
          gozo dev
        </Link>
      </div>
      <Term>
        <Prompt>gozo dev</Prompt>
        <Line mark="step">detected ./cmd/api</Line>
        <Line mark="step">
          loaded .env, .env.local <span className="dim">(6 vars)</span>
        </Line>
        <Line mark="step">
          <b>ready!</b> <span className="link">http://localhost:8080</span>{" "}
          <span className="dim">[1.2s]</span>
        </Line>
        <Line mark="step">
          handler.go changed, rebuilt <span className="dim">[0.4s]</span>
        </Line>
      </Term>
    </section>

    <section className="feature" data-reveal data-x="c">
      <Wire from="l" glyph="●" />
      <div className="copy">
        <h2>one report for the whole workspace.</h2>
        <ul className="points">
          <li>toolchain, go.mod, go.work, tidy and cgo checked together.</li>
          <li>check, test and build run across every module in go.work.</li>
          <li>
            outdated dependencies and stray replace directives called out.
          </li>
        </ul>
        <Link className="more" href="/docs/commands#gozo-doctor">
          gozo doctor
        </Link>
      </div>
      <Term>
        <Prompt>gozo doctor</Prompt>
        <Line mark="ok">go toolchain 1.25.1</Line>
        <Line mark="ok">go 1.25.1 satisfies go directive 1.24</Line>
        <Line mark="ok">go.work covers 3 modules</Line>
        <Line mark="warn">2 dependencies can be updated</Line>
        <Line mark="fail">
          cgo enabled but c compiler <span className="dim">gcc</span> is not on
          path
        </Line>
      </Term>
    </section>

    <figure className="scene" data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <img
        alt="a blurred figure waiting at a crossing under a teal evening sky"
        src="/brand/scenes/crossing.jpg"
      />
      <figcaption>
        <b>the crossing</b>
        <span>waiting on the light with nothing left to carry.</span>
      </figcaption>
    </figure>

    <section className="feature" data-reveal data-x="l">
      <Wire />
      <div className="copy">
        <h2>deploy, then go home.</h2>
        <ul className="points">
          <li>link a directory to docker or kubernetes once.</li>
          <li>deploy, watch logs, roll back to the last good one.</li>
          <li>every deployment recorded, so rollback is one command.</li>
        </ul>
        <Link className="more" href="/docs/deploy">
          deploy targets
        </Link>
      </div>
      <Term>
        <Prompt>gozo rollback</Prompt>
        <Line mark="step">
          rolling api back to 8c1d0f4 <span className="dim">(2h ago)</span>
        </Line>
        <Line mark="ok">
          rollout complete <span className="dim">[9s]</span>
        </Line>
        <Prompt>gozo status</Prompt>
        <Line mark="ok">
          api healthy, 3/3 ready <span className="dim">since 9s</span>
        </Line>
      </Term>
    </section>

    <section className="feature" data-reveal data-x="c">
      <Wire from="l" glyph="●" />
      <div className="copy">
        <h2>built for agents too.</h2>
        <ul className="points">
          <li>every command answers in json with a schema id.</li>
          <li>exit codes mean the same thing everywhere: 0, 1 or 2.</li>
          <li>
            ci and coding agents are detected, so nothing waits on a prompt.
          </li>
        </ul>
        <Link className="more" href="/docs/agents">
          for agents
        </Link>
      </div>
      <Term>
        <Prompt>gozo status --json</Prompt>
        {"{\n"}
        {'  "schema": '}
        <span className="link">&quot;gozo.status/v1&quot;</span>
        {",\n"}
        {'  "project": "api",\n'}
        {'  "target": "kubernetes",\n'}
        {'  "healthy": '}
        <span className="ok">true</span>
        {",\n"}
        {'  "ready": 3,\n'}
        {'  "desired": 3\n'}
        {"}"}
      </Term>
    </section>

    <section data-reveal data-x="l">
      <Wire from="c" glyph="◆" />
      <div className="stats">
        {stats.map((s) => (
          <div key={s.label}>
            <b>{s.value}</b>
            <span>{s.label}</span>
          </div>
        ))}
      </div>
    </section>

    <figure className="scene" data-reveal data-x="l">
      <Wire />
      <img
        alt="two blurred figures walking into warm last light"
        src="/brand/scenes/last-light.jpg"
      />
      <figcaption>
        <b>last light</b>
        <span>that&apos;s how things ship before dinner.</span>
      </figcaption>
    </figure>

    <section className="scatter" data-x="l">
      <Wire />
      <p className="statement">
        <span className="line">{words("you don't need a platform team.")}</span>
        <span className="line">{words("you don't need a makefile.")}</span>
        <span className="line">
          {words("just")}
          <span className="w serif">gozo deploy </span>
          {words("and go home.")}
        </span>
      </p>
    </section>

    <section data-reveal data-x="l" id="install">
      <Wire />
      <div className="install">
        <div className="cmd">
          <span>{INSTALL}</span>
          <CopyButton text={INSTALL} />
        </div>
        <p className="alt">
          or <code>bun add -g gozo</code> · <code>cargo install gozo</code> ·{" "}
          <code>
            curl -fsSL
            https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh |
            sh
          </code>
        </p>
      </div>
    </section>

    <SiteFooter from="l" />
  </div>
);

export default HomePage;
