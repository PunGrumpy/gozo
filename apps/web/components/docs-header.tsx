"use client";

import { useDocsLayout } from "fumadocs-ui/layouts/docs";
import { SidebarIcon } from "lucide-react";

import { Logo } from "./logo";

const iconButton =
  "grid size-10 place-items-center rounded-full p-0 text-gray-1000/80 hover:bg-gray-alpha-100 hover:text-gray-1000 [&_svg]:size-4.5";

/* on phones the site header and Fumadocs' bar would stack as two sticky rows, so this slot is
   both; -me-2.5 puts the menu icon itself, not its 40px hit area, on the header's edge */
export const DocsHeader = () => {
  const { slots } = useDocsLayout();
  const SearchTrigger = slots.searchTrigger ? slots.searchTrigger.sm : null;
  return (
    <header className="bg-background-100/80 max-md:layout:[--fd-header-height:--spacing(16)] sticky top-(--fd-docs-row-1) z-30 flex h-16 items-center border-b border-gray-400 px-6 backdrop-blur-sm [grid-area:header] sm:px-8 md:hidden">
      <Logo />
      <div className="ms-auto -me-2.5 flex items-center">
        {SearchTrigger ? (
          <SearchTrigger className={iconButton} hideIfDisabled />
        ) : null}
        <slots.sidebar.trigger className={iconButton}>
          <SidebarIcon />
        </slots.sidebar.trigger>
      </div>
    </header>
  );
};
