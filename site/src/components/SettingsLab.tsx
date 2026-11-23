import { useState, type ReactNode } from "react";
import { Desktop, DEFAULT_BAR, DEFAULT_DOCK, type BarCfg, type DockCfg } from "./Desktop";
import { Bar, Cpu, Grid, Palette, Windows } from "./Icons";
import { CHARGE_STYLES, TINTS, WALLS, type ChargeStyle, type Tint } from "../lib/theme";

type Page = "dock" | "personalize" | "menubar" | "windows" | "system";

const PAGES: { id: Page; label: string; Icon: typeof Grid }[] = [
  { id: "dock", label: "Dock", Icon: Grid },
  { id: "personalize", label: "Personalize", Icon: Palette },
  { id: "windows", label: "Windows", Icon: Windows },
  { id: "menubar", label: "Menu bar", Icon: Bar },
  { id: "system", label: "System", Icon: Cpu },
];

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-3">
      <div className="min-w-0">
        <div className="text-[13px] font-medium text-foam">{label}</div>
        {hint && <div className="mt-0.5 text-[11.5px] leading-snug text-mute">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Card({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <section className="mb-3 rounded-xl border border-white/[0.07] bg-white/[0.035] px-4">
      {title && <h4 className="pt-3 font-mono text-[10.5px] uppercase tracking-[0.14em] text-faint">{title}</h4>}
      <div className="divide-y divide-white/[0.06]">{children}</div>
    </section>
  );
}

function Seg<T extends string>({
  value,
  onChange,
  options,
  label,
}: {
  value: T;
  onChange: (v: T) => void;
  options: { v: T; l: string }[];
  label: string;
}) {
  return (
    <div role="radiogroup" aria-label={label} className="flex rounded-lg bg-black/40 p-0.5">
      {options.map((o) => (
        <button
          key={o.v}
          role="radio"
          aria-checked={value === o.v}
          onClick={() => onChange(o.v)}
          className={`rounded-md px-2.5 py-1 text-[12px] transition-colors ${
            value === o.v ? "bg-white/15 text-white shadow-sm" : "text-mute hover:text-foam"
          }`}
        >
          {o.l}
        </button>
      ))}
    </div>
  );
}

function Switch({ on, onChange, label }: { on: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button
      role="switch"
      aria-checked={on}
      aria-label={label}
      onClick={() => onChange(!on)}
      className={`relative h-[22px] w-10 rounded-full transition-colors ${on ? "bg-teal" : "bg-white/15"}`}
    >
      <span
        className={`absolute top-[3px] h-4 w-4 rounded-full bg-white shadow transition-[left] ${on ? "left-[21px]" : "left-[3px]"}`}
      />
    </button>
  );
}

function Range({
  value,
  min,
  max,
  onChange,
  label,
  suffix = "",
}: {
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
  label: string;
  suffix?: string;
}) {
  return (
    <div className="flex items-center gap-3">
      <input
        type="range"
        aria-label={label}
        className="wp-range w-36"
        min={min}
        max={max}
        value={value}
        style={{ ["--fill" as string]: `${((value - min) / (max - min)) * 100}%` }}
        onChange={(e) => onChange(Number(e.target.value))}
      />
      <span className="w-12 text-right font-mono text-[11.5px] tabular-nums text-mute">
        {value}
        {suffix}
      </span>
    </div>
  );
}

export function SettingsLab() {
  const [page, setPage] = useState<Page>("menubar");
  const [bar, setBar] = useState<BarCfg>({ ...DEFAULT_BAR });
  const [dock, setDock] = useState<DockCfg>({ ...DEFAULT_DOCK });
  const [wallId, setWallId] = useState(WALLS[0].id);
  const [charge, setCharge] = useState<ChargeStyle>("edge");
  const [fxKey, setFxKey] = useState(0);
  const [autoHide, setAutoHide] = useState(false);
  const [toggles, setToggles] = useState({ music: true, usage: true, bluetooth: false, battery: true, genie: true });
  const wall = WALLS.find((w) => w.id === wallId) ?? WALLS[0];
  const b = <K extends keyof BarCfg>(k: K, v: BarCfg[K]) => setBar((s) => ({ ...s, [k]: v }));
  const d = <K extends keyof DockCfg>(k: K, v: DockCfg[K]) => setDock((s) => ({ ...s, [k]: v }));
  const t = (k: keyof typeof toggles) => (v: boolean) => setToggles((s) => ({ ...s, [k]: v }));

  return (
    <div className="grid items-start gap-6 lg:grid-cols-[minmax(0,560px)_minmax(0,1fr)]">
      {/* The settings window */}
      <div className="overflow-hidden rounded-xl border border-white/10 bg-[#0b1214] shadow-[0_40px_90px_-30px_rgba(0,0,0,0.9)]">
        <div className="flex h-9 items-center justify-between border-b border-white/[0.06] bg-[#091012] pl-3">
          <div className="flex items-center gap-2 text-[12px] text-mute">
            Whirlpool
          </div>
          <div className="flex h-full text-mute" aria-hidden="true">
            {["—", "▢", "✕"].map((c) => (
              <span key={c} className="grid w-11 place-items-center text-[11px]">
                {c}
              </span>
            ))}
          </div>
        </div>
        <div className="grid min-h-[480px] grid-cols-[148px_1fr]">
          <nav aria-label="Settings pages" className="border-r border-white/[0.06] bg-[#091012] p-2">
            {PAGES.map(({ id, label, Icon }) => (
              <button
                key={id}
                onClick={() => setPage(id)}
                aria-current={page === id ? "page" : undefined}
                className={`mb-0.5 flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-[12.5px] transition-colors ${
                  page === id ? "bg-teal/15 text-teal" : "text-mute hover:bg-white/5 hover:text-foam"
                }`}
              >
                <Icon className="h-4 w-4" />
                {label}
              </button>
            ))}
          </nav>
          <div className="max-h-[560px] overflow-y-auto p-4">
            <h3 className="mb-3 font-display text-xl font-bold tracking-tight">
              {PAGES.find((p) => p.id === page)?.label}
            </h3>

            {page === "menubar" && (
              <>
                <Card title="Look">
                  <Row label="Screen edge" hint="Top or bottom. On the bottom the island opens upward.">
                    <Seg
                      label="Bar edge"
                      value={bar.position}
                      onChange={(v) => b("position", v)}
                      options={[
                        { v: "top", l: "Top" },
                        { v: "bottom", l: "Bottom" },
                      ]}
                    />
                  </Row>
                  <Row label="Style" hint="Docked touches the edges. Floating is a rounded bar with a gap.">
                    <Seg
                      label="Bar style"
                      value={bar.style}
                      onChange={(v) => b("style", v)}
                      options={[
                        { v: "docked", l: "Docked" },
                        { v: "floating", l: "Floating" },
                      ]}
                    />
                  </Row>
                  <Row label="Height">
                    <Seg
                      label="Bar height"
                      value={bar.size}
                      onChange={(v) => b("size", v)}
                      options={[
                        { v: "compact", l: "Compact" },
                        { v: "regular", l: "Regular" },
                        { v: "large", l: "Large" },
                      ]}
                    />
                  </Row>
                  <Row label="Opacity">
                    <Range label="Bar opacity" value={bar.opacity} min={10} max={100} suffix=" %" onChange={(v) => b("opacity", v)} />
                  </Row>
                  <div className="py-3">
                    <div className="mb-2 text-[13px] font-medium">Tint</div>
                    <div className="flex flex-wrap gap-2" role="radiogroup" aria-label="Bar tint">
                      {(Object.keys(TINTS) as Tint[]).map((k) => (
                        <button
                          key={k}
                          role="radio"
                          aria-checked={bar.tint === k}
                          aria-label={TINTS[k].label}
                          title={TINTS[k].label}
                          onClick={() => b("tint", k)}
                          className={`h-7 w-7 rounded-full border transition-transform hover:scale-110 ${
                            bar.tint === k ? "border-teal ring-2 ring-teal/40" : "border-white/20"
                          }`}
                          style={{ background: `rgb(${TINTS[k].rgb.map((c) => Math.min(255, c * 2.4 + 18)).join(",")})` }}
                        />
                      ))}
                    </div>
                  </div>
                </Card>
                {bar.style === "floating" && (
                  <Card title="Floating">
                    <Row label="Alignment">
                      <Seg
                        label="Bar alignment"
                        value={bar.align}
                        onChange={(v) => b("align", v)}
                        options={[
                          { v: "left", l: "Left" },
                          { v: "center", l: "Center" },
                          { v: "right", l: "Right" },
                        ]}
                      />
                    </Row>
                    <Row label="Width">
                      <Range label="Bar width" value={bar.width} min={40} max={100} suffix=" %" onChange={(v) => b("width", v)} />
                    </Row>
                  </Card>
                )}
                <Card title="Show">
                  <Row label="Date">
                    <Switch label="Show date" on={bar.date} onChange={(v) => b("date", v)} />
                  </Row>
                  <Row label="Now playing" hint="Song title in the island, with controls.">
                    <Switch label="Now playing" on={toggles.music} onChange={t("music")} />
                  </Row>
                  <Row label="AI plan usage" hint="Quotas and reset times of the coding tools you have open.">
                    <Switch label="AI plan usage" on={toggles.usage} onChange={t("usage")} />
                  </Row>
                  <Row label="Battery percentage">
                    <Switch label="Battery percentage" on={toggles.battery} onChange={t("battery")} />
                  </Row>
                </Card>
              </>
            )}

            {page === "dock" && (
              <>
                <Card title="Position">
                  <Row label="Edge" hint="Bottom, top, left or right.">
                    <Seg
                      label="Dock edge"
                      value={dock.position}
                      onChange={(v) => d("position", v)}
                      options={[
                        { v: "bottom", l: "Bottom" },
                        { v: "left", l: "Left" },
                        { v: "right", l: "Right" },
                        { v: "top", l: "Top" },
                      ]}
                    />
                  </Row>
                  <Row label="Hide automatically" hint="It slides back when you touch the edge.">
                    <Switch label="Auto hide" on={autoHide} onChange={setAutoHide} />
                  </Row>
                </Card>
                <Card title="Icons">
                  <Row label="Size">
                    <Range label="Icon size" value={dock.size} min={36} max={72} suffix=" px" onChange={(v) => d("size", v)} />
                  </Row>
                  <Row label="Magnification" hint="Icons grow under the pointer.">
                    <Switch label="Magnification" on={dock.magnify} onChange={(v) => d("magnify", v)} />
                  </Row>
                </Card>
              </>
            )}

            {page === "personalize" && (
              <>
                <Card title="Wallpaper">
                  <div className="grid grid-cols-4 gap-2 py-3">
                    {WALLS.map((w) => (
                      <button
                        key={w.id}
                        onClick={() => setWallId(w.id)}
                        aria-label={w.label}
                        aria-pressed={wallId === w.id}
                        className={`aspect-[16/10] rounded-lg bg-cover bg-center ring-offset-2 ring-offset-[#0b1214] transition ${
                          wallId === w.id ? "ring-2 ring-teal" : "opacity-80 hover:opacity-100"
                        }`}
                        style={{ backgroundImage: w.css }}
                      />
                    ))}
                  </div>
                </Card>
                <Card title="Charging animation">
                  <div className="grid grid-cols-3 gap-1.5 py-3">
                    {CHARGE_STYLES.map((c) => (
                      <button
                        key={c.id}
                        aria-pressed={charge === c.id}
                        onClick={() => {
                          setCharge(c.id);
                          setFxKey((k) => k + 1);
                        }}
                        className={`rounded-lg border px-2 py-2 text-[12px] transition-colors ${
                          charge === c.id
                            ? "border-teal/60 bg-teal/10 text-teal"
                            : "border-white/10 text-mute hover:text-foam"
                        }`}
                      >
                        {c.name}
                        {c.isNew && <span className="ml-1 font-mono text-[9px] text-teal">NEW</span>}
                      </button>
                    ))}
                  </div>
                  <div className="py-3">
                    <button
                      onClick={() => setFxKey((k) => k + 1)}
                      className="rounded-lg bg-white/10 px-3 py-1.5 text-[12px] text-foam hover:bg-white/15"
                    >
                      Preview
                    </button>
                  </div>
                </Card>
              </>
            )}

            {page === "windows" && (
              <Card title="Windows">
                <Row label="Genie minimize" hint="Windows fold into their dock icon.">
                  <Switch label="Genie minimize" on={toggles.genie} onChange={t("genie")} />
                </Row>
                <Row label="Live thumbnails" hint="Hover an icon to see its windows.">
                  <Switch label="Live thumbnails" on onChange={() => undefined} />
                </Row>
              </Card>
            )}

            {page === "system" && (
              <Card title="System">
                <Row label="Start with Windows">
                  <Switch label="Start with Windows" on onChange={() => undefined} />
                </Row>
                <Row label="Bluetooth in the menu bar">
                  <Switch label="Bluetooth" on={toggles.bluetooth} onChange={t("bluetooth")} />
                </Row>
                <Row label="Language" hint="English, Español, Français.">
                  <span className="rounded-md bg-black/40 px-2.5 py-1 text-[12px] text-mute">English</span>
                </Row>
              </Card>
            )}
          </div>
        </div>
      </div>

      {/* Live preview */}
      <div className="lg:sticky lg:top-24">
        <div className="mb-2 flex items-center justify-between">
          <span className="eyebrow">Live preview</span>
          <span className="font-mono text-[11px] text-faint">Same values the app writes to config.json</span>
        </div>
        <Desktop
          wall={wall}
          bar={bar}
          dock={dock}
          charging={page === "personalize"}
          fx={page === "personalize" ? charge : null}
          fxKey={fxKey}
          percent={page === "personalize" ? 62 : 82}
          className="shadow-[0_40px_90px_-30px_rgba(0,0,0,0.9)]"
        />
      </div>
    </div>
  );
}
