import type { ReactNode } from "react";

export const Term = ({ children }: { children: ReactNode }) => (
  <div className="dark text-gray-1000 relative isolate mx-auto w-full max-w-[720px] overflow-hidden rounded-2xl bg-gray-100 shadow-[inset_0_0_0_1px_var(--ds-gray-alpha-200)] after:pointer-events-none after:absolute after:inset-0 after:bg-[url(/grain.svg)] after:bg-size-[200px] after:opacity-8 after:mix-blend-overlay">
    <pre className="overflow-x-auto px-7.5 py-6.5 font-mono text-sm leading-[1.75] whitespace-pre normal-case [&_b]:font-semibold">
      {children}
    </pre>
  </div>
);

export const Dim = ({ children }: { children: ReactNode }) => (
  <span className="text-gray-900">{children}</span>
);

export const Url = ({ children }: { children: ReactNode }) => (
  <span className="text-blue-900">{children}</span>
);

export const True = () => <span className="text-green-900">true</span>;

export const Prompt = ({ children }: { children: ReactNode }) => (
  <>
    <Dim>$</Dim> {children}
    {"\n"}
  </>
);

const marks = {
  fail: { className: "text-red-900", glyph: "✗" },
  ok: { className: "text-green-900", glyph: "✓" },
  step: { className: "text-gray-900", glyph: ">" },
  warn: { className: "text-amber-900", glyph: "!" },
} as const;

export const Line = ({
  children,
  mark,
}: {
  children: ReactNode;
  mark: keyof typeof marks;
}) => {
  const { className, glyph } = marks[mark];
  return (
    <>
      <span className={className}>{glyph}</span> {children}
      {"\n"}
    </>
  );
};
