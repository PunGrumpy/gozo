"use client";

import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useSyncExternalStore } from "react";

import { cn } from "@/lib/utils";

const themes = [
  { icon: Monitor, id: "system", label: "System theme" },
  { icon: Sun, id: "light", label: "Light theme" },
  { icon: Moon, id: "dark", label: "Dark theme" },
];

/* nothing to subscribe to: the store only tells hydration apart from the client */
const unsubscribe = () => null;
const subscribe = () => unsubscribe;

/* A segmented pill like the site's buttons and Geist's switcher, built on real radios. The
   chosen theme is only known on the client, so nothing is checked until then rather than
   guessing wrong. */
export const ThemeSwitch = () => {
  const { setTheme, theme } = useTheme();
  const mounted = useSyncExternalStore(
    subscribe,
    () => true,
    () => false
  );
  return (
    <div className="flex items-center justify-between text-[13px] text-gray-900">
      <span aria-hidden="true">theme</span>
      <fieldset className="border-gray-alpha-400 inline-flex rounded-full border p-0.5">
        <legend className="sr-only">Theme</legend>
        {themes.map((t) => (
          <label
            className={cn(
              "hover:text-gray-1000 grid size-7 cursor-pointer place-items-center rounded-full text-gray-900 transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none [&_svg]:size-3.5",
              "has-checked:bg-gray-alpha-200 has-checked:text-gray-1000",
              "has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-(--ds-focus-color)"
            )}
            key={t.id}
          >
            <input
              aria-label={t.label}
              checked={mounted && theme === t.id}
              className="sr-only"
              name="theme"
              onChange={() => setTheme(t.id)}
              type="radio"
              value={t.id}
            />
            <t.icon aria-hidden="true" />
          </label>
        ))}
      </fieldset>
    </div>
  );
};
