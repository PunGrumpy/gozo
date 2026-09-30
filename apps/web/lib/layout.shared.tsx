import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { Mark } from "@/components/mark";

import { gitConfig } from "./shared";

export const baseOptions = (): BaseLayoutProps => ({
  githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  links: [
    { active: "none", text: "home", url: "/" },
    { active: "none", text: "brand", url: "/brand" },
  ],
  nav: { title: <Mark /> },
});
