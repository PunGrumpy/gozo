import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/cn";

export const Title = ({
  children,
  className,
  ...props
}: ComponentProps<"h2">) => (
  <h2
    className={cn("text-title max-w-[20ch] font-serif font-normal", className)}
    {...props}
  >
    {children}
  </h2>
);

export const Points = ({ items }: { items: string[] }) => (
  <ul className="border-t border-gray-400">
    {items.map((item) => (
      <li
        className="border-b border-gray-400 py-3 text-[15px] text-gray-900"
        key={item}
      >
        {item}
      </li>
    ))}
  </ul>
);

export const More = ({ className, ...props }: ComponentProps<typeof Link>) => (
  <Link
    className={cn(
      "inline-block font-mono text-[13px] normal-case after:content-['_→'] hover:text-gray-900",
      className
    )}
    {...props}
  />
);

export const Scene = ({
  alt,
  body,
  className,
  file,
  name,
}: {
  alt: string;
  body: ReactNode;
  className?: string;
  file: string;
  name: string;
}) => (
  <figure className="relative">
    <img
      alt={alt}
      className={cn(
        "outline-gray-alpha-300 block aspect-[4/3] w-full rounded-2xl object-cover outline-1 -outline-offset-1",
        className
      )}
      src={`/brand/scenes/${file}.jpg`}
    />
    <figcaption className="absolute right-8 bottom-7 left-8 text-white">
      <b className="block font-serif text-[28px] font-normal tracking-[-0.015em]">
        {name}
      </b>
      <span className="mt-1 block font-mono text-xs opacity-80">{body}</span>
    </figcaption>
  </figure>
);
