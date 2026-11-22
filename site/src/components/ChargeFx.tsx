import type { CSSProperties } from "react";
import type { ChargeStyle } from "../lib/theme";
import { levelColor } from "../lib/theme";
import { Bolt } from "./Icons";

/**
 * The charging animations as CSS: same timings, colours and level thresholds as
 * the Windows build, drawn by the browser instead of the DirectComposition
 * compositor. Sizes use container units (cqw) so the preview scales with its frame.
 */
export function ChargeFx({ style, percent }: { style: ChargeStyle; percent: number }) {
  const color = levelColor(percent);
  const var_ = { "--c": color } as CSSProperties;
  const T = "3.6s";

  switch (style) {
    case "edge":
      return (
        <div className="pointer-events-none absolute inset-0 overflow-hidden rounded-[inherit]" style={var_}>
          <div
            className="absolute inset-0 rounded-[inherit]"
            style={{
              boxShadow: `inset 0 0 7cqw 0.4cqw ${color}, inset 0 0 1.6cqw rgba(255,255,255,0.35), inset 0 0 0 0.12cqw rgba(255,255,255,0.28)`,
              backdropFilter: "blur(0.4px)",
              animation: `fx-edge-in ${T} ease-out both`,
            }}
          />
          <div
            className="absolute inset-0 rounded-[inherit]"
            style={{
              padding: "0.35cqw",
              background: `conic-gradient(from var(--a), transparent 0deg, ${color} 55deg, #fff 75deg, transparent 130deg, transparent 360deg)`,
              WebkitMask: "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
              WebkitMaskComposite: "xor",
              mask: "linear-gradient(#000 0 0) content-box exclude, linear-gradient(#000 0 0)",
              filter: "blur(0.25cqw)",
              animation: `fx-spin ${T} linear both, fx-edge-in ${T} ease-out both`,
            }}
          />
        </div>
      );
    case "tide":
      return (
        <div className="pointer-events-none absolute inset-x-0 top-0 h-[6cqw] overflow-hidden" style={var_}>
          <div className="absolute left-0 top-[1.9cqw] h-[0.5cqw] w-full opacity-15" style={{ background: color }} />
          <div
            className="absolute left-0 top-[1.9cqw] h-[0.5cqw] origin-left"
            style={{
              width: `${Math.max(4, percent)}%`,
              background: `linear-gradient(90deg, transparent, ${color} 30%, ${color})`,
              boxShadow: `0 0 1.6cqw 0.2cqw ${color}`,
              animation: `fx-tide ${T} cubic-bezier(.2,.9,.3,1) both`,
            }}
          >
            <span
              className="absolute -right-[2cqw] -top-[1.8cqw] block h-[4cqw] w-[4cqw] rounded-full"
              style={{ background: `radial-gradient(circle, ${color}, transparent 65%)` }}
            />
          </div>
        </div>
      );
    case "bloom":
      return (
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[34cqw] overflow-hidden" style={var_}>
          <div
            className="absolute inset-0 origin-bottom"
            style={{
              background: `radial-gradient(ellipse 52% 90% at 50% 100%, ${color} 0%, ${color}66 35%, transparent 72%)`,
              filter: "blur(1.2cqw)",
              animation: `fx-bloom ${T} cubic-bezier(.2,.9,.3,1) both`,
            }}
          />
          <div
            className="absolute bottom-0 left-[29%] h-[60%] w-[42%] origin-bottom"
            style={{
              background: `radial-gradient(ellipse 50% 100% at 50% 100%, #fff 0%, ${color} 40%, transparent 75%)`,
              filter: "blur(0.8cqw)",
              animation: `fx-bloom ${T} 0.25s cubic-bezier(.2,.9,.3,1) both`,
            }}
          />
        </div>
      );
    case "aurora": {
      const ribbons = [
        { c: color, w: 95, h: 100, from: "-20%", to: "5%", d: 0 },
        { c: "#4fc3ff", w: 80, h: 76, from: "30%", to: "5%", d: 0.3 },
        { c: "#ffffff", w: 55, h: 50, from: "-5%", to: "40%", d: 0.6 },
      ];
      return (
        <div className="pointer-events-none absolute inset-x-0 top-0 h-[34cqw] overflow-hidden" style={var_}>
          {ribbons.map((r, i) => (
            <div
              key={i}
              className="absolute top-[-4cqw]"
              style={
                {
                  height: `${r.h}%`,
                  width: `${r.w}%`,
                  background: `radial-gradient(ellipse 50% 70% at 50% 0%, ${r.c}${i === 2 ? "88" : "cc"}, transparent 72%)`,
                  filter: "blur(2.2cqw)",
                  mixBlendMode: "screen",
                  "--from": r.from,
                  "--to": r.to,
                  animation: `fx-aurora 4.4s ${r.d}s ease-in-out both`,
                } as CSSProperties
              }
            />
          ))}
        </div>
      );
    }
    case "halo":
      return (
        <div
          className="pointer-events-none absolute left-1/2 top-[3.6cqw] flex h-[4.6cqw] items-center gap-[1.3cqw] rounded-full px-[2.2cqw]"
          style={{
            background: "linear-gradient(180deg, rgba(26,31,38,0.9), rgba(10,13,17,0.9))",
            boxShadow: "inset 0 0 0 0.1cqw rgba(255,255,255,0.14), 0 1.2cqw 3cqw -1cqw rgba(0,0,0,0.7)",
            backdropFilter: "blur(10px)",
            animation: `fx-pill ${T} cubic-bezier(.2,.9,.3,1) both`,
          }}
        >
          <span
            className="relative block h-[1.1cqw] w-[1.1cqw] rounded-full"
            style={{ background: color, boxShadow: `0 0 1.6cqw ${color}` }}
          >
            <span
              className="absolute inset-0 rounded-full"
              style={{ background: color, animation: "fx-breathe 1.8s ease-in-out infinite" }}
            />
          </span>
          <span className="font-pixel text-[1.55cqw] leading-none text-[#eef6f6]">{percent}%</span>
        </div>
      );
    case "surge":
    default:
      return (
        <div className="pointer-events-none absolute left-1/2 top-[3cqw] h-0 w-0" style={var_}>
          {[0, 0.35, 0.7].map((d) => (
            <span
              key={d}
              className="absolute left-0 top-[4.2cqw] block h-[8cqw] w-[8cqw] rounded-full"
              style={{
                border: `0.18cqw solid ${color}`,
                boxShadow: `0 0 1.6cqw ${color}55`,
                animation: `fx-ring 2.6s ${0.5 + d}s ease-out both`,
              }}
            />
          ))}
          <div
            className="absolute left-0 top-0 flex h-[8.4cqw] w-[8.4cqw] flex-col items-center justify-center rounded-full"
            style={{
              background: "linear-gradient(160deg, rgba(34,44,52,0.92), rgba(8,12,16,0.94))",
              boxShadow: `inset 0 0 0 0.12cqw rgba(255,255,255,0.18), 0 0 3cqw -0.4cqw ${color}88, 0 1.5cqw 3cqw -1cqw #000`,
              animation: `fx-medal ${T} cubic-bezier(.2,.9,.3,1) both`,
            }}
          >
            <Bolt className="h-[2.3cqw] w-[2.3cqw]" style={{ color }} />
            <span className="mt-[0.5cqw] font-pixel text-[1.5cqw] leading-none text-[#eef6f6]">{percent}%</span>
            <svg className="absolute inset-0" viewBox="0 0 100 100" aria-hidden="true">
              <circle cx="50" cy="50" r="46" fill="none" stroke="rgba(255,255,255,0.12)" strokeWidth="2.2" />
              <circle
                cx="50"
                cy="50"
                r="46"
                fill="none"
                stroke={color}
                strokeWidth="2.6"
                strokeLinecap="round"
                strokeDasharray={`${(percent / 100) * 289} 289`}
                transform="rotate(-90 50 50)"
              />
            </svg>
          </div>
        </div>
      );
  }
}
