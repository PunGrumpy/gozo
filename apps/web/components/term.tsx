import type { ReactNode } from "react";

export const Term = ({ children }: { children: ReactNode }) => (
  <div className="term">
    <pre>{children}</pre>
  </div>
);

export const Prompt = ({ children }: { children: ReactNode }) => (
  <>
    <span className="dim">$</span> {children}
    {"\n"}
  </>
);

const marks = {
  fail: { cls: "fail", glyph: "✗" },
  ok: { cls: "ok", glyph: "✓" },
  step: { cls: "dim", glyph: ">" },
  warn: { cls: "warn", glyph: "!" },
} as const;

export const Line = ({
  children,
  mark,
}: {
  children: ReactNode;
  mark: keyof typeof marks;
}) => {
  const { cls, glyph } = marks[mark];
  return (
    <>
      <span className={cls}>{glyph}</span> {children}
      {"\n"}
    </>
  );
};
