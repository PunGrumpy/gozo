import { DocsLayout } from "fumadocs-ui/layouts/docs";
import Link from "next/link";

import { DocsHeader } from "@/components/docs-header";
import { REPO, SiteHeader } from "@/components/site";
import { ThemeSwitch } from "@/components/theme-switch";
import { baseOptions } from "@/lib/layout.shared";
import { source } from "@/lib/source";

/* From md up the site header sits above the docs grid. --fd-banner-height tells Fumadocs how tall
   it is, so the sidebar and table of contents stick below it. The grid is 48px narrower than
   the header and padded by 24px, so the sidebar's own 16px inset lands on the header's 40px edge.
   Phones get DocsHeader instead, and the site links move to the foot of the page menu. */
const Layout = ({ children }: LayoutProps<"/docs">) => (
  <div className="md:[--fd-banner-height:4rem]">
    <div className="bg-background-100/80 sticky top-0 z-40 hidden border-b border-gray-400 backdrop-blur-sm md:block">
      <div className="mx-auto max-w-[1280px] px-10">
        <SiteHeader />
      </div>
    </div>
    <DocsLayout
      containerProps={{ className: "[--fd-layout-width:77rem] md:px-6" }}
      sidebar={{
        className: "border-gray-400 bg-background-100 lowercase",
        collapsible: false,
        footer: (
          <>
            <nav aria-label="Site" className="flex gap-5 text-[13px] md:hidden">
              <a className="py-2 hover:text-gray-900" href={REPO}>
                github
              </a>
              <Link className="py-2 hover:text-gray-900" href="/#install">
                install
              </Link>
            </nav>
            <ThemeSwitch />
          </>
        ),
      }}
      slots={{ header: DocsHeader }}
      tree={source.getPageTree()}
      {...baseOptions()}
    >
      {children}
    </DocsLayout>
  </div>
);

export default Layout;
