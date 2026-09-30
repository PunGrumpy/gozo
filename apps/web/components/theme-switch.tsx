"use client";

import { useTheme } from "next-themes";
import { useSyncExternalStore } from "react";

import { cn } from "@/lib/utils";

/* nothing to subscribe to: the store only tells hydration apart from the client */
const unsubscribe = () => null;
const subscribe = () => unsubscribe;

/* two letter boxes in the mark's style. The filled one follows the theme class on <html>, so
   it is right on the first paint; aria-pressed waits until the client knows the theme. */
export const ThemeSwitch = () => {
  const { resolvedTheme, setTheme } = useTheme();
  const mounted = useSyncExternalStore(
    subscribe,
    () => true,
    () => false
  );
  const pressed = (theme: string) =>
    mounted ? resolvedTheme === theme : undefined;
  return (
    <div className="flex items-center justify-between font-mono text-xs text-gray-900 lowercase">
      <span>theme</span>
      <div className="flex gap-1.5">
        <button
          aria-pressed={pressed("light")}
          className={cn(
            "border-gray-1000 text-gray-1000 grid h-6 cursor-pointer place-items-center border px-2 transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
            "border-gray-1000 bg-gray-1000 text-background-100 dark:text-gray-1000 dark:bg-transparent"
          )}
          onClick={() => setTheme("light")}
          type="button"
        >
          light
        </button>
        <button
          aria-pressed={pressed("dark")}
          className={cn(
            "border-gray-1000 text-gray-1000 grid h-6 cursor-pointer place-items-center border px-2 transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
            "dark:bg-gray-1000 dark:text-background-100"
          )}
          onClick={() => setTheme("dark")}
          type="button"
        >
          dark
        </button>
      </div>
    </div>
  );
};
