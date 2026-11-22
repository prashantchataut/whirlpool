import { useCallback, useRef, useState, type ReactNode, type SVGProps } from "react";
import { TINTS, rgba, type ChargeStyle, type Tint, type Wall } from "../lib/theme";
import { ChargeFx } from "./ChargeFx";
import {
  Battery,
  Bluetooth,
  Bolt,
  Download,
  Folder,
  Gear,
  Globe,
  Mail,
  Music,
  Search,
  Terminal,
  Trash,
  Volume,
  Wifi,
} from "./Icons";

export type BarCfg = {
  position: "top" | "bottom";
  style: "docked" | "floating";
  size: "compact" | "regular" | "large";
  tint: Tint;
  opacity: number;
  align: "left" | "center" | "right";
  width: number;
  date: boolean;
};
export type DockCfg = {
  position: "bottom" | "left" | "right" | "top";
  size: number;
  magnify: boolean;
};

export const DEFAULT_BAR: BarCfg = {
  position: "top",
  style: "docked",
  size: "regular",
  tint: "ocean",
  opacity: 50,
  align: "center",
  width: 100,
  date: true,
};
export const DEFAULT_DOCK: DockCfg = { position: "bottom", size: 52, magnify: true };

/** Reference screen width: all sizes below are written for 1440 px and scale with the frame. */
const REF = 14.4;
const cq = (px: number) => `${(px / REF).toFixed(3)}cqw`;

const BAR_H = { compact: 24, regular: 28, large: 34 } as const;

const APPS: { name: string; Icon: (p: SVGProps<SVGSVGElement>) => ReactNode; a: string; b: string; open?: boolean }[] = [
  { name: "Files", Icon: Folder, a: "#4fb8d8", b: "#1f6f94", open: true },
  { name: "Browser", Icon: Globe, a: "#5ee0b5", b: "#13765f", open: true },
  { name: "Terminal", Icon: Terminal, a: "#4b5a66", b: "#171d22" },
  { name: "Music", Icon: Music, a: "#f08a8a", b: "#9b2f45" },
  { name: "Mail", Icon: Mail, a: "#7aa7ff", b: "#2a4bb0", open: true },
  { name: "Settings", Icon: Gear, a: "#aab6c0", b: "#56626d" },
  { name: "Downloads", Icon: Download, a: "#ffc766", b: "#b5651b" },
  { name: "Recycle Bin", Icon: Trash, a: "#8da0ad", b: "#3d4a54" },
];

export type DesktopProps = {
  wall: Wall;
  bar?: BarCfg;
  dock?: DockCfg;
  percent?: number;
  charging?: boolean;
  fx?: ChargeStyle | null;
  fxKey?: number;
  showDock?: boolean;
  children?: ReactNode;
  className?: string;
  rounded?: string;
};

export function Desktop({
  wall,
  bar = DEFAULT_BAR,
  dock = DEFAULT_DOCK,
  percent = 82,
  charging = false,
  fx = null,
  fxKey = 0,
  showDock = true,
  children,
  className = "",
  rounded = "rounded-2xl",
}: DesktopProps) {
  return (
    <div
      className={`relative aspect-[16/10] w-full overflow-hidden bg-ink-2 ${rounded} ${className}`}
      style={{ containerType: "inline-size" }}
    >
      <div
        className="absolute inset-0 bg-cover bg-center transition-[background-image] duration-500"
        style={{ backgroundImage: wall.css }}
      />
      <div className="absolute inset-0 bg-gradient-to-b from-black/10 via-transparent to-black/30" />
      <MenuBar cfg={bar} percent={percent} charging={charging} />
      {showDock && <Dock cfg={dock} barBottom={bar.style === "floating"} barPos={bar.position} barH={BAR_H[bar.size] + (bar.style === "floating" ? 8 : 0)} />}
      {children}
      {fx && <ChargeFx key={`${fx}-${fxKey}-${percent}`} style={fx} percent={percent} />}
      <div className="pointer-events-none absolute inset-0 rounded-[inherit] ring-1 ring-inset ring-white/10" />
    </div>
  );
}

/* ----------------------------------------------------------- Menu bar */

function MenuBar({ cfg, percent, charging }: { cfg: BarCfg; percent: number; charging: boolean }) {
  const h = BAR_H[cfg.size];
  const floating = cfg.style === "floating";
  const inset = floating ? 24 : 0;
  const gap = floating ? 8 : 0;
  const width = floating ? cfg.width : 100;
  const atBottom = cfg.position === "bottom";
  const base: React.CSSProperties = {
    ...(atBottom ? { bottom: cq(gap) } : { top: cq(gap) }),
    height: cq(h),
    fontSize: cq(h * 0.46),
    background: rgba(TINTS[cfg.tint].rgb, cfg.opacity / 100),
    backdropFilter: "blur(14px) saturate(1.4)",
    WebkitBackdropFilter: "blur(14px) saturate(1.4)",
    borderRadius: floating ? cq(14) : 0,
    boxShadow: floating
      ? "inset 0 0 0 1px rgba(255,255,255,0.12), 0 10px 30px -12px rgba(0,0,0,0.6)"
      : atBottom
        ? "inset 0 1px 0 rgba(255,255,255,0.1)"
        : "inset 0 -1px 0 rgba(255,255,255,0.1)",
    transition:
      "top .35s cubic-bezier(.2,.9,.3,1), bottom .35s cubic-bezier(.2,.9,.3,1), height .35s cubic-bezier(.2,.9,.3,1), width .35s cubic-bezier(.2,.9,.3,1), left .35s cubic-bezier(.2,.9,.3,1), border-radius .35s, background .3s",
    width: `calc(${width}% - ${cq(inset * 2 * (width / 100))})`,
    left:
      cfg.align === "left"
        ? cq(inset)
        : cfg.align === "right"
          ? `calc(100% - ${width}% + ${cq(inset * (width / 100))})`
          : `calc(${(100 - width) / 2}% + ${cq(inset * (width / 100))})`,
  };
  if (!floating) {
    base.left = 0;
    base.width = "100%";
  }
  const icon = { width: cq(h * 0.52), height: cq(h * 0.52) };
  return (
    <div className="absolute z-20 flex items-center justify-between px-[0.9cqw] text-white/90" style={base}>
      <div className="flex min-w-0 items-center" style={{ gap: cq(14) }}>
        <span className="font-semibold">Whirlpool</span>
        <span className="hidden text-white/65 sm:inline">File</span>
        <span className="hidden text-white/65 sm:inline">View</span>
        <span className="hidden text-white/65 md:inline">Window</span>
      </div>
      <div
        className="absolute left-1/2 flex -translate-x-1/2 items-center justify-center rounded-full bg-black font-medium tabular-nums"
        style={{ height: cq(h * 0.74), minWidth: cq(96), fontSize: cq(h * 0.4) }}
      >
        9:41
      </div>
      <div className="flex items-center" style={{ gap: cq(11) }}>
        <Search style={icon} />
        <Bluetooth style={icon} />
        <Wifi style={icon} />
        <Volume style={icon} />
        <span className="flex items-center" style={{ gap: cq(4) }}>
          <span className="tabular-nums">{percent}%</span>
          <Battery level={percent / 100} style={{ width: cq(h * 0.9), height: cq(h * 0.45) }} />
          {charging && <Bolt style={{ width: cq(h * 0.38), height: cq(h * 0.38), color: "#47e67a" }} />}
        </span>
        {cfg.date && <span className="hidden tabular-nums text-white/80 md:inline">Fri 14 Nov</span>}
      </div>
    </div>
  );
}

/* --------------------------------------------------------------- Dock */

function Dock({ cfg, barBottom, barPos, barH }: { cfg: DockCfg; barBottom: boolean; barPos: "top" | "bottom"; barH: number }) {
  const list = useRef<HTMLDivElement>(null);
  const [pointer, setPointer] = useState<number | null>(null);
  const vertical = cfg.position === "left" || cfg.position === "right";
  const base = cfg.size;
  const gapPx = Math.max(4, base * 0.16);

  const onMove = useCallback(
    (e: React.PointerEvent) => {
      const box = list.current?.getBoundingClientRect();
      if (!box) return;
      setPointer(vertical ? e.clientY - box.top : e.clientX - box.left);
    },
    [vertical],
  );

  const total = APPS.length;
  const box = list.current?.getBoundingClientRect();
  const span = box ? (vertical ? box.height : box.width) : 1;
  const pitch = span / total;

  const place: React.CSSProperties =
    cfg.position === "bottom"
      ? { bottom: cq(10 + (barPos === "bottom" ? barH : 0)), left: "50%", transform: "translateX(-50%)" }
      : cfg.position === "top"
        ? { top: cq(barPos === "top" ? (barBottom ? 44 : 38) + 4 : 14), left: "50%", transform: "translateX(-50%)" }
        : cfg.position === "left"
          ? { left: cq(10), top: "52%", transform: "translateY(-50%)" }
          : { right: cq(10), top: "52%", transform: "translateY(-50%)" };

  const origin =
    cfg.position === "bottom" ? "bottom" : cfg.position === "top" ? "top" : cfg.position === "left" ? "left" : "right";

  return (
    <div
      className="absolute z-10 glass-strong"
      style={{
        ...place,
        padding: cq(7),
        borderRadius: cq(base * 0.38),
        background: "linear-gradient(180deg, rgba(255,255,255,0.16), rgba(255,255,255,0.07))",
        transition: "all .35s cubic-bezier(.2,.9,.3,1)",
      }}
      onPointerMove={onMove}
      onPointerLeave={() => setPointer(null)}
    >
      <div
        ref={list}
        className="flex"
        style={{ flexDirection: vertical ? "column" : "row", gap: cq(gapPx) }}
      >
        {APPS.map((app, i) => {
          let s = 1;
          if (cfg.magnify && pointer !== null) {
            const center = (i + 0.5) * pitch;
            const d = (pointer - center) / (pitch * 1.5);
            s = 1 + 0.72 * Math.exp(-d * d);
          }
          return (
            <div
              key={app.name}
              className="relative"
              style={{
                width: cq(base),
                height: cq(base),
                transform: `scale(${s.toFixed(3)})`,
                transformOrigin: origin,
                transition: pointer === null ? "transform .35s cubic-bezier(.2,.9,.3,1)" : "transform 60ms linear",
              }}
              title={app.name}
            >
              <div
                className="grid h-full w-full place-items-center text-white"
                style={{
                  borderRadius: cq(base * 0.26),
                  background: `linear-gradient(155deg, ${app.a}, ${app.b})`,
                  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.35), inset 0 -8px 14px -8px rgba(0,0,0,0.35), 0 4px 10px -4px rgba(0,0,0,0.6)",
                }}
              >
                <app.Icon style={{ width: cq(base * 0.5), height: cq(base * 0.5) }} />
              </div>
              {app.open && (
                <span
                  className="absolute left-1/2 block -translate-x-1/2 rounded-full bg-white/90"
                  style={{
                    width: cq(4),
                    height: cq(4),
                    ...(cfg.position === "top"
                      ? { top: cq(-7) }
                      : cfg.position === "left"
                        ? { left: cq(-7), top: "50%", transform: "translateY(-50%)" }
                        : cfg.position === "right"
                          ? { right: cq(-7), left: "auto", top: "50%", transform: "translateY(-50%)" }
                          : { bottom: cq(-7) }),
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
