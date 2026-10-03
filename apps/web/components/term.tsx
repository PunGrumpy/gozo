import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

export const Term = ({ children }: { children: ReactNode }) => (
  <div className="border-gray-alpha-400 text-gray-1000 relative isolate mx-auto w-full max-w-[720px] overflow-hidden rounded-2xl border bg-gray-100 after:pointer-events-none after:absolute after:inset-0 after:bg-[url(/grain.svg)] after:bg-size-[200px] after:opacity-8 after:mix-blend-overlay">
    <pre className="px-5 py-5 font-mono text-[13px] leading-[1.75] wrap-break-word whitespace-pre-wrap normal-case lg:px-7.5 lg:py-6.5 lg:text-sm [&_b]:font-semibold">
      {children}
    </pre>
  </div>
);

export const Dim = ({ children }: { children: ReactNode }) => (
  <span className="whitespace-nowrap text-gray-900">{children}</span>
);

export const Url = ({ children }: { children: ReactNode }) => (
  <span className="text-blue-900">{children}</span>
);

export const True = () => <span className="text-green-900">true</span>;

/* the mark hangs in a two-character gutter, so a line that wraps on a phone continues
   under its text rather than under the mark */
const Row = ({
  children,
  glyph,
  tone,
}: {
  children: ReactNode;
  glyph: string;
  tone: string;
}) => (
  <span className="block pl-[2ch] -indent-[2ch]">
    <span className={cn("inline-block w-[1ch] indent-0", tone)}>{glyph}</span>{" "}
    {children}
  </span>
);

export const Prompt = ({ children }: { children: ReactNode }) => (
  <Row glyph="$" tone="text-gray-900">
    {children}
  </Row>
);

const marks = {
  fail: { glyph: "✗", tone: "text-red-900" },
  ok: { glyph: "✓", tone: "text-green-900" },
  step: { glyph: ">", tone: "text-gray-900" },
  warn: { glyph: "!", tone: "text-amber-900" },
} as const;

export const Line = ({
  children,
  mark,
}: {
  children: ReactNode;
  mark: keyof typeof marks;
}) => <Row {...marks[mark]}>{children}</Row>;
