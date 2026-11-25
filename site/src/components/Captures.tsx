const base = import.meta.env.BASE_URL;

type Shot = { file: string; title: string; note: string; wide?: boolean };

const shots: Shot[] = [
  {
    file: "hud-volume.jpg",
    title: "Volume",
    note: "Gradient fill, knob and number. The fill glides to the new level.",
  },
  {
    file: "hud-muted.jpg",
    title: "Muted",
    note: "Crossed speaker, grey fill, dimmed number.",
  },
  {
    file: "hud-brightness.jpg",
    title: "Brightness",
    note: "Drawn on demand with whirlpool --hud. No hardware event feeds it yet.",
  },
  {
    file: "hud-wifi.jpg",
    title: "Wi-Fi",
    note: "Appears when Wi-Fi connects or drops. Four signal steps, no text to translate.",
  },
  {
    file: "hud-bluetooth.jpg",
    title: "Bluetooth",
    note: "A switch that follows the radio, reported by Windows as it changes.",
  },
];

const screens: Shot[] = [
  {
    file: "start-grid.jpg",
    title: "Start grid",
    note: "Blurred picture of the screen behind it, glass icon plates. Some programs only have a low-resolution icon and show it as is.",
    wide: true,
  },
  {
    file: "quick-settings.jpg",
    title: "Quick settings",
    note: "The dialog behind it is the runner's own. Wi-Fi and Bluetooth read \u201cNot available\u201d because a hosted runner has neither.",
    wide: true,
  },
  {
    file: "settings.jpg",
    title: "Settings",
    note: "The new slider is the one beside Size. The sidebar folds to icons at this 1024-pixel width.",
    wide: true,
  },
];

function Figure({ s }: { s: Shot }) {
  return (
    <figure className="glass overflow-hidden rounded-2xl">
      <img
        src={`${base}captures/${s.file}`}
        alt={`${s.title}, captured from whirlpool.exe on a Windows runner`}
        loading="lazy"
        className="w-full"
      />
      <figcaption className="p-4">
        <h3 className="text-[14.5px] font-semibold text-foam">{s.title}</h3>
        <p className="mt-1 text-[13px] leading-relaxed text-mute">{s.note}</p>
      </figcaption>
    </figure>
  );
}

export function Captures() {
  return (
    <div className="grid gap-8">
      <p className="max-w-3xl text-[14.5px] leading-relaxed text-mute">
        Every image here is a screenshot of whirlpool.exe running on a GitHub-hosted Windows Server 2025 runner at
        1024 x 768, 100% scale, taken by the capture driver in the repository. None is a render. The scenario list and
        what still needs a real desktop are in TESTING.md.
      </p>
      <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
        {shots.map((s) => (
          <Figure key={s.file} s={s} />
        ))}
      </div>
      <div className="grid gap-5 lg:grid-cols-3">
        {screens.map((s) => (
          <Figure key={s.file} s={s} />
        ))}
      </div>
    </div>
  );
}
