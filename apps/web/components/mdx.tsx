import { CodeBlock, Pre } from "fumadocs-ui/components/codeblock";
import { Heading } from "fumadocs-ui/components/heading";
import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";

const serif = "font-serif font-normal tracking-[-0.02em]";

export const getMDXComponents = (components?: MDXComponents) =>
  ({
    ...defaultMdxComponents,
    h2: (props) => (
      <Heading as="h2" className={`${serif} text-[1.75rem]`} {...props} />
    ),
    h3: (props) => <Heading as="h3" className={serif} {...props} />,
    /* code blocks are the terminal: a .dark subtree, so they stay dark on a light page */
    pre: (props) => (
      <div className="dark">
        <CodeBlock
          {...props}
          className="text-gray-1000 rounded-2xl border-0 bg-gray-100 shadow-[inset_0_0_0_1px_var(--ds-gray-alpha-200)]"
        >
          <Pre>{props.children}</Pre>
        </CodeBlock>
      </div>
    ),
    ...components,
  }) satisfies MDXComponents;

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
