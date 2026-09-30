"use client";

import { useState } from "react";

import { cn } from "@/lib/utils";

const RESET_MS = 1600;

/* both labels share one grid cell, so the button never changes width; the swap crossfades through a 2px blur */
export const CopyButton = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);
  return (
    <button
      aria-live="polite"
      className={cn(
        "inline-flex shrink-0 items-center justify-center gap-2 rounded-full border font-medium whitespace-nowrap transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
        "h-10 px-5 text-sm",
        "bg-gray-1000 text-background-100 hover:bg-gray-1000/85 border-transparent",
        /* both labels share one grid cell, so the width never changes */
        "grid min-w-22 cursor-pointer font-[inherit]"
      )}
      onClick={async () => {
        await navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), RESET_MS);
      }}
      type="button"
    >
      <span
        className="col-start-1 row-start-1 transition-[opacity,filter] duration-200 data-hidden:opacity-0 data-hidden:blur-[2px] motion-reduce:transition-none"
        data-hidden={copied ? "" : undefined}
      >
        copy
      </span>
      <span
        aria-hidden={!copied}
        className="col-start-1 row-start-1 transition-[opacity,filter] duration-200 data-hidden:opacity-0 data-hidden:blur-[2px] motion-reduce:transition-none"
        data-hidden={copied ? undefined : ""}
      >
        copied
      </span>
    </button>
  );
};
