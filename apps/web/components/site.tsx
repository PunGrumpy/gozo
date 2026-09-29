import Link from "next/link";

export const REPO = "https://github.com/PunGrumpy/gozo";

/* one wire per block: a grey guide, the ink stroke drawn as the block passes the middle of the viewport, an optional jog from the previous x and a node at the joint */
export const Wire = ({ from, glyph }: { from?: "l" | "c"; glyph?: string }) => (
  <>
    <i className="wire">
      <b />
    </i>
    {from ? (
      <i className="jog" data-from={from}>
        <b />
      </i>
    ) : null}
    {glyph ? (
      <i aria-hidden="true" className="node">
        {glyph}
      </i>
    ) : null}
  </>
);

export const SiteHeader = () => (
  <header>
    <Link className="k" href="/">
      <span aria-hidden="true" className="on">
        g
      </span>
      <span aria-hidden="true">o</span>
      <span aria-hidden="true">z</span>
      <span aria-hidden="true">o</span>
      <span className="sr-only">gozo</span>
    </Link>
    <nav>
      <Link href="/docs">docs</Link>
      <Link href="/brand">brand</Link>
      <a href={REPO}>github</a>
      <Link className="btn" href="/#install">
        install
      </Link>
    </nav>
  </header>
);

export const SiteFooter = ({ from }: { from?: "l" | "c" }) => (
  <footer data-x="c">
    <Wire from={from} glyph="■" />
    <div className="links">
      <span>mit license</span>
      <span>
        <a href={REPO}>github</a> · <Link href="/brand">brand</Link> ·{" "}
        <a href="https://www.npmjs.com/package/gozo">npm</a>
      </span>
    </div>
    <span className="sr-only">gozo</span>
    <i aria-hidden="true" className="node end">
      ●
    </i>
    <div aria-hidden="true" className="wm" />
  </footer>
);
