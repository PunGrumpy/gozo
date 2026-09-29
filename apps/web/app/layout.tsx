import "./global.css";
import { RootProvider } from "fumadocs-ui/provider/next";
import type { Metadata, Viewport } from "next";
import { Geist, Geist_Mono, Instrument_Serif } from "next/font/google";

const geist = Geist({ subsets: ["latin"], variable: "--font-geist" });
const geistMono = Geist_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
});
const instrumentSerif = Instrument_Serif({
  style: ["normal", "italic"],
  subsets: ["latin"],
  variable: "--font-instrument-serif",
  weight: "400",
});

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
    className={`${geist.variable} ${geistMono.variable} ${instrumentSerif.variable} font-sans`}
    lang="en"
    suppressHydrationWarning
  >
    <body className="flex min-h-screen flex-col">
      <RootProvider>{children}</RootProvider>
    </body>
  </html>
);

export default Layout;
