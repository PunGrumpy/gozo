"use client";

import { useState } from "react";

const RESET_MS = 1600;

export const CopyButton = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);
  return (
    <button
      aria-live="polite"
      className="landing-copy"
      onClick={async () => {
        await navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), RESET_MS);
      }}
      type="button"
    >
      <span data-hidden={copied ? "" : undefined}>copy</span>
      <span aria-hidden={!copied} data-hidden={copied ? undefined : ""}>
        copied
      </span>
    </button>
  );
};
