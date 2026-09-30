import "./styles/globals.css";
import { RootProvider } from "fumadocs-ui/provider/next";
import type { Metadata, Viewport } from "next";

import { fonts } from "@/lib/fonts";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  description:
    "gozo is one binary that gives Go projects the workflow of Vercel CLI: dev, env, deploy, logs, rollback, doctor, check, test, build. JSON on every command.",
  metadataBase: new URL("https://gozo.pungrumpy.com"),
  title: { default: "gozo · the joy of go, handled.", template: "%s · gozo" },
};

export const viewport: Viewport = {
  themeColor: [
    { color: "hsl(240 20% 99%)", media: "(prefers-color-scheme: light)" },
    { color: "hsl(240 8% 4%)", media: "(prefers-color-scheme: dark)" },
  ],
};

const Layout = ({ children }: LayoutProps<"/">) => (
  <html
    className={cn(
      fonts,
      "bg-background-100 scrollbar-gutter-stable scroll-smooth"
    )}
    lang="en"
    suppressHydrationWarning
    data-scroll-behavior="smooth"
  >
    <body className="flex min-h-screen flex-col">
      <RootProvider>{children}</RootProvider>
    </body>
  </html>
);

export default Layout;
