import {
  Geist as createSans,
  Geist_Mono as createMono,
  Instrument_Serif as createSerif,
} from "next/font/google";

import { cn } from "./utils";

const sans = createSans({
  display: "swap",
  subsets: ["latin"],
  variable: "--font-sans",
  weight: "variable",
});

const mono = createMono({
  display: "swap",
  subsets: ["latin"],
  variable: "--font-mono",
  weight: "variable",
});

const serif = createSerif({
  style: ["normal", "italic"],
  subsets: ["latin"],
  variable: "--font-serif",
  weight: "400",
});

export const fonts = cn(
  "touch-manipulation font-sans antialiased [font-synthesis-weight:none]",
  sans.variable,
  mono.variable,
  serif.variable
);
