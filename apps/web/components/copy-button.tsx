"use client";

import { useState } from "react";

import { button } from "@/lib/button";
import { cn } from "@/lib/utils";

const RESET_MS = 1600;

/* both labels share one grid cell, so the button never changes width; the swap crossfades through a 2px blur */
const label =
  "col-start-1 row-start-1 transition-[opacity,filter] duration-200 data-hidden:opacity-0 data-hidden:blur-[2px] motion-reduce:transition-none";

export const CopyButton = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);
  return (
    <button
      aria-live="polite"
      className={cn(
        button(),
        "grid min-w-22 shrink-0 cursor-pointer place-items-center font-[inherit]"
      )}
      onClick={async () => {
        await navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), RESET_MS);
      }}
      type="button"
    >
      <span className={label} data-hidden={copied ? "" : undefined}>
        copy
      </span>
      <span
        aria-hidden={!copied}
        className={label}
        data-hidden={copied ? undefined : ""}
      >
        copied
      </span>
    </button>
  );
};
