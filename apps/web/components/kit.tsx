const pieces = [
  { id: "gomod", label: "go.mod" },
  { id: "dev", label: "gozo dev" },
  { id: "build", label: "gozo build" },
  { id: "env", label: "gozo env" },
  { id: "deploy", label: "gozo deploy" },
  { id: "status", label: "gozo status" },
  { id: "gozo", label: "gozo" },
  { id: "one", label: "gozo" },
];

/* the kit starts scattered across the stage and folds into one box as you scroll; scattered positions live in kit.generated.css, the motion in landing.css */
export const Kit = () => (
  <div className="kit">
    {pieces.map((p) => (
      <figure className="piece" data-id={p.id} key={p.id}>
        <img alt={`${p.label}, drawn as line art`} src={`/kit/${p.id}.svg`} />
        <figcaption>{p.label}</figcaption>
      </figure>
    ))}
  </div>
);
