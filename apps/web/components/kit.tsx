import { cn } from "@/lib/utils";

import { art } from "./kit-art";

/* Positions in percent of the 1200 × 540 stage; o is the piece's turn in the build. Each
   brick sits in the box; fx/fy is where its piece stands relative to that slot and ox/oy
   is the brick's own centre, so it scales around itself. */
const pieces = [
  {
    fx: -38.58,
    fy: -26.94,
    id: "gomod",
    label: "go.mod",
    ox: "50.01%",
    oy: "62.18%",
    sx: 3.34,
    sy: 11.3,
    w: 16.15,
  },
  {
    fx: -20.69,
    fy: 9.63,
    id: "dev",
    label: "gozo dev",
    ox: "46.51%",
    oy: "34.97%",
    sx: 20.51,
    sy: 31.67,
    w: 16.15,
  },
  {
    fx: -8.02,
    fy: -22.13,
    id: "build",
    label: "gozo build",
    ox: "72.74%",
    oy: "54.4%",
    sx: 41.12,
    sy: 14.44,
    w: 11.1,
  },
  {
    fx: 1.36,
    fy: 11.76,
    id: "env",
    label: "gozo env",
    ox: "27.27%",
    oy: "49.22%",
    sx: 42.7,
    sy: 46.3,
    w: 7.93,
  },
  {
    fx: 17.39,
    fy: -1.11,
    id: "deploy",
    label: "gozo deploy",
    ox: "53.51%",
    oy: "68.65%",
    sx: 61.12,
    sy: 37.04,
    w: 13.99,
  },
  {
    fx: 34.28,
    fy: -19.44,
    id: "status",
    label: "gozo status",
    ox: "50.01%",
    oy: "40.41%",
    sx: 78.07,
    sy: 10,
    w: 12.41,
  },
  {
    fx: 33.53,
    fy: 40.64,
    id: "gozo",
    label: "gozo",
    ox: "49.07%",
    oy: "37.31%",
    sx: 78.5,
    sy: 74.07,
    w: 9.67,
  },
] as const;

const box = { sx: 39.68, sy: 27.13, w: 20.63 };

const place = (p: { sx: number; sy: number; w: number }) => ({
  left: `${p.sx}cqw`,
  top: `${p.sy}cqh`,
  width: `${p.w}cqw`,
});

const Art = ({ id, label }: { id: keyof typeof art; label?: string }) => (
  <svg
    aria-hidden={label ? undefined : true}
    aria-label={label}
    className="fill-background-100 text-gray-1000 block h-auto w-full"
    role={label ? "img" : undefined}
    viewBox={art[id].viewBox}
  >
    {art[id].body}
  </svg>
);

/* Built like lego, one piece per turn, bottom layer first: the piece blurs into its brick,
   the brick flies over its slot and presses down into place. When the last one lands the
   seams fuse into one box. Without scroll timelines, with reduced motion or on a phone,
   only the finished box shows. */
export const Kit = () => (
  <div className="[container-type:size] relative mb-6 aspect-[1200/540] w-full [--lift:6cqh] [--step:9svh] md:max-h-[44svh]">
    {pieces.map((p, o) => (
      <figure
        className="md:scroll-motion:lift-off absolute opacity-0"
        key={p.id}
        style={{ ...place(p), "--o": o }}
      >
        <Art id={p.id} label={`${p.label}, drawn as line art`} />
        <figcaption
          className={cn(
            "absolute top-full left-1/2 -translate-x-1/2 pt-3.5 font-mono text-xs tracking-[0.04em] whitespace-nowrap text-gray-900 before:absolute before:top-0.5 before:left-1/2 before:h-2 before:w-px before:bg-gray-900"
          )}
        >
          {p.label}
        </figcaption>
      </figure>
    ))}
    {pieces.map((p, o) => (
      <div
        className="md:scroll-motion:snap absolute opacity-0"
        key={p.id}
        style={{
          ...place(box),
          "--fx": p.fx,
          "--fy": p.fy,
          "--o": o,
          transformOrigin: `${p.ox} ${p.oy}`,
          zIndex: o + 1,
        }}
      >
        <Art id={`brick-${p.id}`} />
      </div>
    ))}
    <figure
      className="md:scroll-motion:fade-in-scroll absolute z-20 [--from:calc(7.2*var(--step))] [--to:calc(7.8*var(--step))]"
      style={place(box)}
    >
      <Art id="one" label="gozo, drawn as line art" />
      <figcaption
        className={cn(
          "absolute top-full left-1/2 -translate-x-1/2 pt-3.5 font-mono text-xs tracking-[0.04em] whitespace-nowrap text-gray-900 before:absolute before:top-0.5 before:left-1/2 before:h-2 before:w-px before:bg-gray-900",
          "md:scroll-motion:fade-in-scroll [--from:calc(7.8*var(--step))] [--to:calc(8.6*var(--step))]"
        )}
      >
        gozo
      </figcaption>
    </figure>
  </div>
);
