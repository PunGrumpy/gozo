/* positions in percent of the 1200 × 540 stage. sx/sy/w place a piece; o is its turn in the
   build; each brick sits in the box and starts over its piece (fx, fy), turning around ox/oy */
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
];

const box = { sx: 39.68, sy: 27.13, w: 20.63 };

const place = (p: { sx: number; sy: number; w: number }) => ({
  left: `${p.sx}cqw`,
  top: `${p.sy}cqh`,
  width: `${p.w}cqw`,
});

const art = "block h-auto w-full dark:invert dark:hue-rotate-180";
const caption =
  "absolute top-full left-1/2 -translate-x-1/2 pt-3.5 font-mono text-[11px] tracking-[0.04em] whitespace-nowrap text-gray-900 before:absolute before:top-0.5 before:left-1/2 before:h-2 before:w-px before:bg-gray-900";

/* on scroll each piece lifts away and its brick drops into the gozo box. Without scroll
   timelines, with reduced motion or on a phone, only the finished box shows. */
export const Kit = () => (
  <div className="[container-type:size] relative mb-6 aspect-[1200/540] w-full [--lift:12cqh] [--step:8svh] md:max-h-[44svh]">
    {pieces.map((p, o) => (
      <figure
        className="md:scroll-motion:lift-off absolute opacity-0"
        key={p.id}
        style={{ ...place(p), "--o": o }}
      >
        <img
          alt={`${p.label}, drawn as line art`}
          className={art}
          src={`/kit/${p.id}.svg`}
        />
        <figcaption className={caption}>{p.label}</figcaption>
      </figure>
    ))}
    {pieces.map((p, o) => (
      <img
        alt=""
        className={`md:scroll-motion:snap absolute opacity-0 ${art}`}
        key={p.id}
        src={`/kit/brick-${p.id}.svg`}
        style={{
          ...place(box),
          "--fx": p.fx,
          "--fy": p.fy,
          "--o": o,
          transformOrigin: `${p.ox} ${p.oy}`,
          zIndex: o + 1,
        }}
      />
    ))}
    <figure
      className="md:scroll-motion:fade-in-scroll absolute z-20 [--from:66svh] [--to:72svh]"
      style={place(box)}
    >
      <img alt="gozo, drawn as line art" className={art} src="/kit/one.svg" />
      <figcaption
        className={`${caption} md:scroll-motion:fade-in-scroll [--from:76svh] [--to:86svh]`}
      >
        gozo
      </figcaption>
    </figure>
  </div>
);
