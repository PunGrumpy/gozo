import Link from "next/link";
import type { ReactNode } from "react";

import { button } from "@/lib/button";
import { cn } from "@/lib/cn";

import { Mark } from "./mark";
import { More, Points, Title } from "./ui";

export const REPO = "https://github.com/PunGrumpy/gozo";

type Side = "l" | "c";

const node =
  "absolute left-(--x) z-1 grid size-4 place-items-center border border-gray-1000 bg-background-100 font-mono text-[8px] leading-none text-gray-1000 not-italic";

/* one wire per block: a grey guide at --x, inked as the block crosses the middle of the
   viewport, a jog from the previous block's side, and a node at the joint */
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
        "pointer-events-none absolute top-0 bottom-0 left-(--x) -z-1 w-px bg-gray-400",
        lineClassName
      )}
    >
      <b className="bg-gray-1000 scroll-motion:draw-y absolute inset-0 origin-top" />
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
      <i aria-hidden="true" className={cn(node, "top-0 -translate-1/2")}>
        {glyph}
      </i>
    ) : null}
  </>
);

/* a block hangs off the wire on the left (l) or at the centre (c); on phones everything sits left */
export const Block = ({
  as: Tag = "section",
  children,
  className,
  from,
  glyph,
  id,
  reveal = true,
  x,
}: {
  as?: "section" | "div";
  children: ReactNode;
  className?: string;
  from?: Side;
  glyph?: string;
  id?: string;
  reveal?: boolean;
  x: Side;
}) => (
  <Tag
    className={cn(
      "relative [--x:0%]",
      x === "c" && "md:[--x:50%]",
      Tag === "section" && "py-20 md:py-30",
      reveal &&
        "scroll-motion:[&>:not(i)]:reveal scroll-motion:[&>:not(i):nth-child(n+3)]:reveal-late",
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

const navLink = "inline-block py-2 hover:text-gray-900";

export const SiteHeader = () => (
  <header className="flex items-center justify-between py-4 text-[13px]">
    <Link href="/">
      <Mark lit />
    </Link>
    <nav className="flex items-center gap-4 md:gap-6">
      <Link className={navLink} href="/docs">
        docs
      </Link>
      <Link className={navLink} href="/brand">
        brand
      </Link>
      <a className={navLink} href={REPO}>
        github
      </a>
      <Link className={button({ small: true })} href="/#install">
        install
      </Link>
    </nav>
  </header>
);

/* the wire stops at a node just above the wordmark. The footer is a container, so the
   wordmark's visible height (--wm) follows its width. */
export const SiteFooter = ({ from }: { from?: Side }) => (
  <footer className="@container relative pt-30 [--wm:calc(100cqw*160/242*0.78)] [--x:0%] md:[--x:50%]">
    <Wire from={from} glyph="■" lineClassName="bottom-(--wm)" />
    <div className="flex justify-between pb-10 text-[13px]">
      <span>mit license</span>
      <span>
        <a className={navLink} href={REPO}>
          github
        </a>{" "}
        ·{" "}
        <Link className={navLink} href="/brand">
          brand
        </Link>{" "}
        ·{" "}
        <a className={navLink} href="https://www.npmjs.com/package/gozo">
          npm
        </a>
      </span>
    </div>
    <span className="sr-only">gozo</span>
    <i
      aria-hidden="true"
      className={cn(node, "bottom-(--wm) -translate-x-1/2 translate-y-1/2")}
    >
      ●
    </i>
    <div
      aria-hidden="true"
      className="bg-gray-1000 h-(--wm) w-full mask-[url(/brand/wordmark-ink.svg)] mask-size-[100%_auto] mask-top mask-no-repeat"
    />
  </footer>
);
