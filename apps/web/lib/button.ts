import { cn } from "./utils";

/* one pill for every call to action: hover changes colour instantly, a press scales */
export const button = ({
  ghost = false,
  small = false,
}: { ghost?: boolean; small?: boolean } = {}) =>
  cn(
    "inline-flex items-center gap-2 rounded-full border leading-none transition-transform duration-160 ease-out active:scale-97 motion-reduce:transition-none",
    small ? "px-3.5 py-2 text-[13px]" : "px-4.5 py-2.5 text-sm",
    ghost
      ? "text-gray-1000 hover:border-gray-1000 border-gray-400"
      : "border-gray-1000 bg-gray-1000 text-background-100"
  );
