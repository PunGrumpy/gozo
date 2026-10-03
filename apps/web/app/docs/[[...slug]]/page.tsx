import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  MarkdownCopyButton,
  ViewOptionsPopover,
} from "fumadocs-ui/layouts/docs/page";
import { createRelativeLink } from "fumadocs-ui/mdx";
import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { getMDXComponents } from "@/components/mdx";
import { getPageImageUrl, getPageMarkdownUrl, gitConfig } from "@/lib/shared";
import { source } from "@/lib/source";
import { cn } from "@/lib/utils";

const Page = async (props: PageProps<"/docs/[[...slug]]">) => {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) {
    notFound();
  }

  const Mdx = page.data.body;
  const markdownUrl = getPageMarkdownUrl(page).url;

  return (
    <DocsPage
      className="px-6 sm:px-8 md:px-10 xl:px-10"
      full={page.data.full}
      tableOfContentPopover={{
        content: { className: "px-2 sm:px-4" },
        trigger: { className: "px-6 sm:px-8 md:px-10" },
      }}
      toc={page.data.toc}
    >
      <DocsTitle className="font-serif text-[2.75rem] leading-none font-normal tracking-[-0.02em]">
        {page.data.title}
      </DocsTitle>
      <DocsDescription className="mb-0 text-gray-900">
        {page.data.description}
      </DocsDescription>
      <div className="flex flex-row items-center gap-2 border-b border-gray-400 pb-6">
        <MarkdownCopyButton
          className={cn(
            "inline-flex shrink-0 items-center justify-center gap-2 rounded-full border font-medium whitespace-nowrap transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
            "h-8 px-3.5 text-[13px]",
            "border-gray-alpha-400 text-gray-1000 hover:bg-gray-alpha-100 bg-transparent"
          )}
          markdownUrl={markdownUrl}
        />
        <ViewOptionsPopover
          className={cn(
            "inline-flex shrink-0 items-center justify-center gap-2 rounded-full border font-medium whitespace-nowrap transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
            "h-8 px-3.5 text-[13px]",
            "border-gray-alpha-400 text-gray-1000 hover:bg-gray-alpha-100 bg-transparent"
          )}
          markdownUrl={markdownUrl}
          githubUrl={`https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/content/docs/${page.path}`}
        />
      </div>
      <DocsBody className="[&_:not(pre)>code]:whitespace-nowrap">
        <Mdx
          components={getMDXComponents({
            // this allows you to link to other pages with relative file paths
            a: createRelativeLink(source, page),
          })}
        />
      </DocsBody>
    </DocsPage>
  );
};

export default Page;

export const generateStaticParams = () => source.generateParams();

export const generateMetadata = async (
  props: PageProps<"/docs/[[...slug]]">
): Promise<Metadata> => {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) {
    notFound();
  }

  return {
    description: page.data.description,
    openGraph: {
      images: getPageImageUrl(page).url,
    },
    title: page.data.title,
  };
};
