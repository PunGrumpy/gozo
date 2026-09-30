import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

/* the site header carries the mark and the links; the docs sidebar keeps search and its own theme switch */
export const baseOptions = (): BaseLayoutProps => ({
  nav: { title: <span className="sr-only">gozo docs</span> },
  themeSwitch: { enabled: false },
});
