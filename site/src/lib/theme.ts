import ocean from "../assets/wall-ocean.jpg";
import dusk from "../assets/wall-dusk.jpg";
import lake from "../assets/wall-lake.jpg";
import ridge from "../assets/wall-ridge.jpg";

export const REPO = "https://github.com/prashantchataut/whirlpool";
export const VERSION = "4.0.0";
export const SETUP_URL = `${REPO}/releases/latest/download/Whirlpool-Setup.exe`;
export const ZIP_URL = `${REPO}/releases/latest/download/Whirlpool-x64.zip`;
export const RELEASES_URL = `${REPO}/releases/latest`;

/** The menu bar tints, exactly the dark-mode values the Windows build uses. */
export type Tint =
  | "ocean"
  | "graphite"
  | "midnight"
  | "forest"
  | "amber"
  | "rose"
  | "violet"
  | "mint"
  | "sunset"
  | "slate"
  | "sand";

export const TINTS: Record<Tint, { label: string; rgb: [number, number, number] }> = {
  ocean: { label: "Ocean", rgb: [6, 28, 36] },
  graphite: { label: "Graphite", rgb: [22, 23, 26] },
  midnight: { label: "Midnight", rgb: [10, 14, 38] },
  forest: { label: "Forest", rgb: [8, 30, 20] },
  amber: { label: "Amber", rgb: [42, 26, 8] },
  rose: { label: "Rose", rgb: [40, 12, 24] },
  violet: { label: "Violet", rgb: [26, 14, 44] },
  mint: { label: "Mint", rgb: [8, 38, 32] },
  sunset: { label: "Sunset", rgb: [46, 18, 10] },
  slate: { label: "Slate", rgb: [20, 26, 34] },
  sand: { label: "Sand", rgb: [38, 33, 24] },
};

export type Wall = { id: string; label: string; css: string; credit?: string };

export const WALLS: Wall[] = [
  { id: "ocean", label: "Shoreline", css: `url(${ocean})`, credit: "Nala Berting / Pexels" },
  { id: "dusk", label: "Dusk swell", css: `url(${dusk})`, credit: "Mete Kaan Özdilek / Pexels" },
  { id: "lake", label: "First light", css: `url(${lake})`, credit: "Quang Nguyen Vinh / Pexels" },
  { id: "ridge", label: "Fog ridge", css: `url(${ridge})`, credit: "Quang Nguyen Vinh / Pexels" },
];

export type ChargeStyle = "surge" | "halo" | "edge" | "tide" | "bloom" | "aurora";

export const CHARGE_STYLES: { id: ChargeStyle; name: string; blurb: string; isNew?: boolean }[] = [
  { id: "surge", name: "Surge", blurb: "A glass medallion drops in with the level, and rings of water spread out from it." },
  { id: "halo", name: "Halo", blurb: "One small glass pill under the menu bar. A dot that breathes, the number, nothing else." },
  { id: "edge", name: "Edge", blurb: "The rim of the screen glows like liquid glass, with a slow highlight running around it." },
  { id: "tide", name: "Tide", blurb: "A thin line of light runs along the top edge and stops at your battery level." },
  { id: "bloom", name: "Bloom", blurb: "A soft glow rises behind the dock and settles back down.", isNew: true },
  { id: "aurora", name: "Aurora", blurb: "Three slow ribbons of colour drift across the top of the screen.", isNew: true },
];

/** Same thresholds as the app: green when healthy, yellow in the middle, orange when low. */
export function levelColor(percent: number): string {
  if (percent <= 20) return "#ff8a1f";
  if (percent <= 59) return "#ffcc2e";
  return "#47e67a";
}

export function rgba([r, g, b]: [number, number, number], a: number): string {
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}
