import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { gitConfig } from "./shared";

export const baseOptions = (): BaseLayoutProps => ({
  githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  links: [
    { text: "docs", url: "/docs" },
    { text: "brand", url: "/brand" },
  ],
  nav: {
    title: (
      <span className="flex items-center gap-2">
        <img
          alt=""
          height={22}
          src="/brand/symbol-ink.svg"
          width={22}
          className="dark:invert"
        />
        <span className="font-serif text-2xl tracking-[-0.02em]">gozo</span>
      </span>
    ),
  },
});
