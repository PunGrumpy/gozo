"use client";

import { ContextMenu } from "@base-ui/react/context-menu";
import { BookOpen, Check, Image, Sprout, Type } from "lucide-react";
import Link from "next/link";
import { useState } from "react";

import { cn } from "@/lib/utils";

import { Lockup } from "./lockup";

const assets = [
  {
    file: "/brand/logo-ink.svg",
    icon: Image,
    id: "logo",
    label: "copy logo as svg",
  },
  {
    file: "/brand/symbol-ink.svg",
    icon: Sprout,
    id: "symbol",
    label: "copy symbol as svg",
  },
  {
    file: "/brand/wordmark-ink.svg",
    icon: Type,
    id: "wordmark",
    label: "copy wordmark as svg",
  },
];

/* A click on the lockup goes home; a right click or long press opens the brand menu, the
   way Vercel and Orchid do. The popup grows from the pointer and leaves faster than it comes. */
export const Logo = () => {
  const [copied, setCopied] = useState<string | null>(null);
  const copy = async (asset: (typeof assets)[number]) => {
    const response = await fetch(asset.file);
    await navigator.clipboard.writeText(await response.text());
    setCopied(asset.id);
  };
  return (
    <ContextMenu.Root
      onOpenChange={(open) => {
        if (!open) {
          setCopied(null);
        }
      }}
    >
      <ContextMenu.Trigger
        render={
          <Link
            aria-label="Gozo home"
            className="text-gray-1000 -mx-1.5 inline-flex rounded-md px-1.5 py-1"
            href="/"
          />
        }
      >
        <Lockup className="h-7 w-auto" />
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Positioner className="z-50 outline-hidden">
          <ContextMenu.Popup
            className={cn(
              "border-gray-alpha-400 text-gray-1000 min-w-60 origin-(--transform-origin) rounded-xl border p-1 font-sans lowercase outline-hidden transition-[scale,opacity] duration-150 ease-out data-ending-style:scale-96 data-ending-style:opacity-0 data-ending-style:duration-100 data-starting-style:scale-96 data-starting-style:opacity-0 motion-reduce:transition-none",
              /* elevation: a black shadow in both themes, and in dark a surface one step lighter than the page, since a shadow barely reads on a dark page */
              "bg-background-100 shadow-lg shadow-black/8 dark:bg-gray-100 dark:shadow-black/40"
            )}
          >
            {assets.map((asset) => {
              const Icon = copied === asset.id ? Check : asset.icon;
              return (
                <ContextMenu.Item
                  className="text-gray-1000 data-highlighted:bg-gray-alpha-100 flex h-9 cursor-default items-center gap-3 rounded-lg px-3 text-sm outline-hidden select-none [&_svg]:size-4 [&_svg]:text-gray-900"
                  closeOnClick={false}
                  key={asset.id}
                  onClick={() => copy(asset)}
                >
                  <Icon aria-hidden="true" />
                  {copied === asset.id ? "copied" : asset.label}
                </ContextMenu.Item>
              );
            })}
            <ContextMenu.Separator className="bg-gray-alpha-400 -mx-1 my-1 h-px" />
            <ContextMenu.LinkItem
              className="text-gray-1000 data-highlighted:bg-gray-alpha-100 flex h-9 cursor-default items-center gap-3 rounded-lg px-3 text-sm outline-hidden select-none [&_svg]:size-4 [&_svg]:text-gray-900"
              render={<Link href="/brand" />}
            >
              <BookOpen aria-hidden="true" />
              brand guidelines
            </ContextMenu.LinkItem>
          </ContextMenu.Popup>
        </ContextMenu.Positioner>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
};
