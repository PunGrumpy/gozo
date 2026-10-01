import { createGetUrl } from "fumadocs-core/source";

export const appName = "gozo";
export const docsRoute = "/docs";
export const docsImageRoute = "/og/docs";
export const docsContentRoute = "/llms.mdx/docs";

export const gitConfig = {
  branch: "main",
  repo: "gozo",
  user: "PunGrumpy",
};

const getContentUrl = createGetUrl(docsContentRoute);

export const getPageMarkdownUrl = (page: {
  slugs: string[];
  locale?: string;
}) => {
  const segments = [...page.slugs, "content.md"];
  return { segments, url: getContentUrl(segments, page.locale) };
};

const getImageUrl = createGetUrl(docsImageRoute);

export const getPageImageUrl = (page: { slugs: string[]; locale?: string }) => {
  const segments = [...page.slugs, "image.png"];
  return { segments, url: getImageUrl(segments, page.locale) };
};
