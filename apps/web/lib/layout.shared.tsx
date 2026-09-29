import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { gitConfig } from "./shared";

export const baseOptions = (): BaseLayoutProps => ({
  githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  links: [
    { active: "none", text: "home", url: "/" },
    { active: "none", text: "brand", url: "/brand" },
  ],
  nav: {
    title: (
      <span className="k">
        <span aria-hidden="true" className="on">
          g
        </span>
        <span aria-hidden="true">o</span>
        <span aria-hidden="true">z</span>
        <span aria-hidden="true">o</span>
        <span className="sr-only">gozo</span>
      </span>
    ),
  },
});
