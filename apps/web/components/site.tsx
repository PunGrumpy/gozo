import Link from "next/link";
import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

import { Logo } from "./logo";
import { More, Points, Title } from "./ui";

export const REPO = "https://github.com/PunGrumpy/gozo";

type Side = "l" | "c";

/* one wire per block: a grey guide at --x, inked as the block crosses the middle of the
   viewport, a jog from the previous block's side drawn first, and a node at the joint */
const Wire = ({
  from,
  glyph,
  lineClassName,
}: {
  from?: Side;
  glyph?: string;
  lineClassName?: string;
}) => (
  <>
    <i
      aria-hidden="true"
      className={cn(
        "pointer-events-none absolute top-0 bottom-0 left-(--x) -z-1 w-px bg-gray-400 max-md:hidden",
        lineClassName
      )}
    >
      <b
        className={cn(
          "bg-gray-1000 scroll-motion:draw-y absolute inset-0 origin-top",
          from && "[--wire-from:cover_6%]"
        )}
      />
    </i>
    {from ? (
      <i
        aria-hidden="true"
        className="pointer-events-none absolute top-0 left-0 -z-1 hidden h-px w-1/2 bg-gray-400 md:block"
      >
        <b
          className={cn(
            "bg-gray-1000 scroll-motion:draw-x absolute inset-0",
            from === "l" ? "origin-left" : "origin-right"
          )}
        />
      </i>
    ) : null}
    {glyph ? (
      <i
        aria-hidden="true"
        className={cn(
          "border-gray-1000 bg-background-100 text-gray-1000 absolute left-(--x) z-1 grid size-4 place-items-center border font-mono text-[8px] leading-none not-italic",
          "top-0 -translate-1/2 max-md:hidden"
        )}
      >
        {glyph}
      </i>
    ) : null}
  </>
);

/* a block hangs off the wire on the left (l) or at the centre (c); phones drop the wire and everything sits left */
export const Block = ({
  as: Tag = "section",
  children,
  className,
  from,
  glyph,
  id,
  x,
}: {
  as?: "section" | "div";
  children: ReactNode;
  className?: string;
  from?: Side;
  glyph?: string;
  id?: string;
  x: Side;
}) => (
  <Tag
    className={cn(
      "relative [--x:0%]",
      x === "c" && "md:[--x:50%]",
      Tag === "section" && "py-20 md:py-30",
      className
    )}
    id={id}
  >
    <Wire from={from} glyph={glyph} />
    {children}
  </Tag>
);

export const Feature = ({
  children,
  from,
  glyph,
  more,
  points,
  title,
  x,
}: {
  children: ReactNode;
  from?: Side;
  glyph?: string;
  more?: { href: string; label: string };
  points: string[];
  title: string;
  x: Side;
}) => (
  <Block
    className="grid items-center gap-8 md:grid-cols-2 md:gap-16"
    from={from}
    glyph={glyph}
    x={x}
  >
    <div
      className={cn(
        "max-w-[480px] pl-(--gutter)",
        x === "c" && "md:order-1 md:pl-0"
      )}
    >
      <Title className="mb-5">{title}</Title>
      <Points items={points} />
      {more ? (
        <More className="mt-5 py-2" href={more.href}>
          {more.label}
        </More>
      ) : null}
    </div>
    {children}
  </Block>
);

const cue = [..."scroll"].map((char, i) => ({
  char,
  filled: i % 2 === 0,
  id: `${char}${i}`,
}));

/* the boxed "scroll" under a hero, where the wire starts */
export const ScrollCue = () => (
  <div
    aria-hidden="true"
    className="absolute bottom-12 left-1/2 hidden -translate-x-1/2 gap-1.5 md:flex"
  >
    {cue.map((c) => (
      <span
        className={cn(
          "border-gray-1000 grid size-5 place-items-center border font-mono text-[10px]",
          c.filled && "bg-gray-1000 text-background-100"
        )}
        key={c.id}
      >
        {c.char}
      </span>
    ))}
  </div>
);

export const SiteHeader = () => (
  <header className="flex h-16 items-center justify-between lowercase">
    <Logo />
    <nav className="flex items-center gap-1">
      <Link
        className="text-gray-1000/80 hover:bg-gray-alpha-100 hover:text-gray-1000 inline-flex items-center rounded-lg px-3 py-2 text-sm leading-none"
        href="/docs"
      >
        docs
      </Link>
      <a
        className="text-gray-1000/80 hover:bg-gray-alpha-100 hover:text-gray-1000 inline-flex items-center rounded-lg px-3 py-2 text-sm leading-none"
        href={REPO}
      >
        github
      </a>
      <Link
        className={cn(
          "inline-flex shrink-0 items-center justify-center gap-2 rounded-full border font-medium whitespace-nowrap transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
          "h-8 px-3.5 text-[13px]",
          "bg-gray-1000 text-background-100 hover:bg-gray-1000/85 border-transparent",
          "ml-2"
        )}
        href="/#install"
      >
        install
      </Link>
    </nav>
  </header>
);

export const SiteFooter = ({ from }: { from?: Side }) => (
  <footer className="@container relative pt-30 [--wm:calc(100cqw*160/242*0.78)] [--x:0%] md:[--x:50%]">
    <Wire from={from} glyph="■" lineClassName="bottom-(--wm)" />
    <div className="flex justify-between pb-10 text-[13px]">
      <Link
        href="https://www.pungrumpy.com"
        className="inline-block py-2 hover:text-gray-900"
      >
        Noppakorn Kaewsalabnil
      </Link>
      <span>
        <a className="inline-block py-2 hover:text-gray-900" href={REPO}>
          github
        </a>{" "}
        ·{" "}
        <Link className="inline-block py-2 hover:text-gray-900" href="/brand">
          brand
        </Link>{" "}
        ·{" "}
        <a
          className="inline-block py-2 hover:text-gray-900"
          href="https://www.npmjs.com/package/gozo-cli"
        >
          npm
        </a>
      </span>
    </div>
    <span className="sr-only">gozo</span>
    <i
      aria-hidden="true"
      className={cn(
        "border-gray-1000 bg-background-100 text-gray-1000 absolute left-(--x) z-1 grid size-4 place-items-center border font-mono text-[8px] leading-none not-italic",
        "bottom-(--wm) -translate-x-1/2 translate-y-1/2 max-md:hidden"
      )}
    >
      ●
    </i>
    <div
      aria-hidden="true"
      className="bg-gray-1000 h-(--wm) w-full mask-[url(/brand/wordmark-ink.svg)] mask-size-[100%_auto] mask-top mask-no-repeat"
    />
  </footer>
);
