import { DocsLayout } from "fumadocs-ui/layouts/docs";

import { SiteHeader } from "@/components/site";
import { ThemeSwitch } from "@/components/theme-switch";
import { baseOptions } from "@/lib/layout.shared";
import { source } from "@/lib/source";

/* The site header sits above the docs grid. --fd-banner-height tells Fumadocs how tall it
   is, so the sidebar and table of contents stick below it. The grid is 48px narrower than
   the header and padded by 24px, so the sidebar's own 16px inset lands on the header's 40px edge. */
const Layout = ({ children }: LayoutProps<"/docs">) => (
  <div className="[--fd-banner-height:4rem]">
    <div className="bg-background-100/80 sticky top-0 z-40 border-b border-gray-400 backdrop-blur-sm">
      <div className="mx-auto max-w-[1280px] px-6 sm:px-8 md:px-10">
        <SiteHeader />
      </div>
    </div>
    <DocsLayout
      containerProps={{ className: "[--fd-layout-width:77rem] md:px-6" }}
      sidebar={{
        className: "border-gray-400 bg-background-100 lowercase",
        collapsible: false,
        footer: <ThemeSwitch />,
      }}
      tree={source.getPageTree()}
      {...baseOptions()}
    >
      {children}
    </DocsLayout>
  </div>
);

export default Layout;
