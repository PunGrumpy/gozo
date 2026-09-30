const pieces = [
  { id: "gomod", label: "go.mod" },
  { id: "dev", label: "gozo dev" },
  { id: "build", label: "gozo build" },
  { id: "env", label: "gozo env" },
  { id: "deploy", label: "gozo deploy" },
  { id: "status", label: "gozo status" },
  { id: "gozo", label: "gozo" },
];

/* on scroll each piece turns into a brick of the gozo box and snaps into place; positions live in kit.generated.css, the motion in landing.css */
export const Kit = () => (
  <div className="kit">
    {pieces.map((p) => (
      <figure className="piece" data-id={p.id} key={p.id}>
        <img alt={`${p.label}, drawn as line art`} src={`/kit/${p.id}.svg`} />
        <figcaption>{p.label}</figcaption>
      </figure>
    ))}
    {pieces.map((p) => (
      <img
        alt=""
        className="brick"
        data-id={p.id}
        key={p.id}
        src={`/kit/brick-${p.id}.svg`}
      />
    ))}
    <figure className="piece" data-id="one">
      <img alt="gozo, drawn as line art" src="/kit/one.svg" />
      <figcaption>gozo</figcaption>
    </figure>
  </div>
);
