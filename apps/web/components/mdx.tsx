import { CodeBlock, Pre } from "fumadocs-ui/components/codeblock";
import { Heading } from "fumadocs-ui/components/heading";
import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";

import { cn } from "@/lib/utils";

export const getMDXComponents = (components?: MDXComponents) =>
  ({
    ...defaultMdxComponents,
    /* on wide screens each section hangs a node on the sidebar's edge, the docs' wire */
    h2: (props) => (
      <Heading
        as="h2"
        className={cn(
          "relative font-serif text-[1.75rem] font-normal tracking-[-0.02em]",
          "xl:before:border-gray-1000 xl:before:bg-background-100 xl:before:text-gray-1000 xl:before:absolute xl:before:top-1/2 xl:before:-left-10 xl:before:z-40 xl:before:grid xl:before:size-4 xl:before:-translate-1/2 xl:before:place-items-center xl:before:border xl:before:font-mono xl:before:text-[8px] xl:before:leading-none xl:before:content-['◆']"
        )}
        {...props}
      />
    ),
    h3: (props) => (
      <Heading
        as="h3"
        className="font-serif font-normal tracking-[-0.02em]"
        {...props}
      />
    ),
    /* Code blocks follow the theme like the landing's terminals. With a mouse, the copy
       button only shows on hover or focus so it never sits on top of a long line; touch
       keeps it visible. */
    pre: (props) => (
      <CodeBlock
        {...props}
        className="border-gray-alpha-400 rounded-2xl bg-gray-100 shadow-none pointer-fine:[&>div:first-child]:opacity-0 pointer-fine:focus-within:[&>div:first-child]:opacity-100 pointer-fine:hover:[&>div:first-child]:opacity-100"
        viewportProps={{ className: "pe-12" }}
      >
        <Pre>{props.children}</Pre>
      </CodeBlock>
    ),
    ...components,
  }) satisfies MDXComponents;

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
