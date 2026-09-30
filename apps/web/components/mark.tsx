import { cn } from "@/lib/utils";

const letters = [
  { char: "g", id: "g" },
  { char: "o", id: "o1" },
  { char: "z", id: "z" },
  { char: "o", id: "o2" },
];

/* the four letter boxes; with lit, each box takes its turn filled as the page scrolls */
export const Mark = ({ lit = false }: { lit?: boolean }) => (
  <span className="inline-flex gap-1.5">
    {letters.map((l, i) => (
      <span
        aria-hidden="true"
        className={cn(
          "border-gray-1000 text-gray-1000 grid size-6 place-items-center border font-mono text-[11px] normal-case",
          i === 0 && "bg-gray-1000 text-background-100",
          lit && "scroll-motion:lit",
          lit &&
            i === 0 &&
            "scroll-motion:bg-transparent scroll-motion:text-gray-1000"
        )}
        key={l.id}
        style={{ "--i": i }}
      >
        {l.char}
      </span>
    ))}
    <span className="sr-only">gozo</span>
  </span>
);
