import { Fragment, useRef, useState, type ReactNode } from 'react'
import logo from '../assets/logo.png'

export type DockPosition = 'bottom' | 'left' | 'right'
export type AnimationStyle = 'relaxed' | 'smooth' | 'snappy'
export type StartupAnimation = 'off' | 'glide' | 'rise' | 'swift'

export const STARTUP_MS: Record<StartupAnimation, number> = { off: 0, glide: 1150, rise: 720, swift: 380 }
const FOLLOW_MS: Record<AnimationStyle, number> = { relaxed: 200, smooth: 110, snappy: 55 }
const SETTLE_MS: Record<AnimationStyle, number> = { relaxed: 520, smooth: 320, snappy: 180 }

type Glyph = { id: string; label: string; from: string; to: string; art: ReactNode; running?: boolean }

const stroke = { fill: 'none', stroke: '#fff', strokeWidth: 1.8, strokeLinecap: 'round', strokeLinejoin: 'round' } as const

const APPS: Glyph[] = [
  { id: 'whirlpool', label: 'Whirlpool', from: '', to: '', art: null, running: true },
  {
    id: 'files', label: 'Files', from: '#ffd36a', to: '#ff9a3d', running: true,
    art: <path {...stroke} d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />,
  },
  {
    id: 'browser', label: 'Browser', from: '#6fc3ff', to: '#3567e8', running: true,
    art: (
      <g {...stroke}>
        <circle cx="12" cy="12" r="8.5" />
        <path d="M3.5 12h17M12 3.5c3.2 3.4 3.2 13.6 0 17M12 3.5c-3.2 3.4-3.2 13.6 0 17" />
      </g>
    ),
  },
  { id: 'terminal', label: 'Terminal', from: '#3a4a5c', to: '#10171f', art: <path {...stroke} d="M5.5 8.5 9.5 12l-4 3.5M12.5 16.5h6" /> },
  {
    id: 'music', label: 'Music', from: '#ff7ab6', to: '#e02f6e',
    art: (
      <g {...stroke}>
        <path d="M9 17.5V6.5l9-2v11" />
        <circle cx="7" cy="17.5" r="2" />
        <circle cx="16" cy="15.5" r="2" />
      </g>
    ),
  },
  {
    id: 'mail', label: 'Mail', from: '#7be0c3', to: '#16a085',
    art: (
      <g {...stroke}>
        <rect x="3.5" y="5.5" width="17" height="13" rx="2.5" />
        <path d="m4 8 8 5.5L20 8" />
      </g>
    ),
  },
  {
    id: 'photos', label: 'Photos', from: '#c79bff', to: '#6a4fe0',
    art: (
      <g {...stroke}>
        <rect x="3.5" y="4.5" width="17" height="15" rx="3" />
        <circle cx="9" cy="10" r="1.6" />
        <path d="m4 17.5 5-4.5 3.5 3 3-2.5 4.5 4" />
      </g>
    ),
  },
]
const BIN: Glyph = {
  id: 'bin', label: 'Recycle Bin', from: '#b9c6d4', to: '#6e8196',
  art: <path {...stroke} d="M5 7.5h14M9.5 7.5v-2h5v2M7 7.5l.8 11h8.4l.8-11M10.5 11v5M13.5 11v5" />,
}

function Tile({ g, size }: { g: Glyph; size: number }) {
  const radius = size * 0.26
  if (g.id === 'whirlpool')
    return <img src={logo} alt="" draggable={false} style={{ width: size, height: size }} className="block" />
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: radius,
        background: `linear-gradient(160deg, ${g.from}, ${g.to})`,
        boxShadow: 'inset 0 1px 0 rgba(255,255,255,.55), inset 0 -8px 14px rgba(0,0,0,.18), 0 6px 14px -6px rgba(0,0,0,.55)',
      }}
      className="grid place-items-center"
    >
      <svg viewBox="0 0 24 24" style={{ width: size * 0.56, height: size * 0.56 }}>{g.art}</svg>
    </div>
  )
}

export type DockProps = {
  position?: DockPosition
  size?: number
  magnified?: number
  magnify?: boolean
  style?: AnimationStyle
  startup?: StartupAnimation
  replayKey?: number
  charging?: boolean
  className?: string
}

/** The dock, as the app draws it: spring magnification, bounce, indicators, startup rise. */
export default function Dock({
  position = 'bottom', size = 52, magnified = 84, magnify = true, style = 'smooth',
  startup = 'rise', replayKey = 0, charging = false, className = '',
}: DockProps) {
  const anchor = useRef<HTMLDivElement>(null)
  const [pointer, setPointer] = useState<number | null>(null)
  const [bounce, setBounce] = useState<string | null>(null)
  const [hover, setHover] = useState<string | null>(null)
  const vertical = position !== 'bottom'
  const items = [...APPS, BIN]
  const gap = Math.max(6, size * 0.14)
  const sepAfter = APPS.length // separator before the bin
  const sepSlot = gap + 1

  // Rest layout centres, measured from the dock's own (stable) centre.
  const total = items.length * size + (items.length - 1) * gap + sepSlot
  const restCentre = (i: number) => -total / 2 + i * (size + gap) + size / 2 + (i >= sepAfter ? sepSlot : 0)

  const sizeOf = (i: number) => {
    if (!magnify || pointer === null) return size
    const d = Math.abs(pointer - restCentre(i))
    const range = size * 2.6
    if (d >= range) return size
    return size + (magnified - size) * Math.cos((d / range) * (Math.PI / 2)) ** 2
  }

  const onMove = (e: React.MouseEvent) => {
    const r = anchor.current?.getBoundingClientRect()
    if (!r) return
    setPointer(vertical ? e.clientY - (r.top + r.height / 2) : e.clientX - (r.left + r.width / 2))
  }

  const flow = vertical ? 'flex-col' : 'flex-row'
  const origin = position === 'bottom' ? 'items-end' : position === 'left' ? 'items-start' : 'items-end'
  const ms = pointer === null ? SETTLE_MS[style] : FOLLOW_MS[style]
  const dur = STARTUP_MS[startup]

  return (
    <div
      ref={anchor}
      className={`pointer-events-none flex ${vertical ? 'h-full flex-col justify-center' : 'w-full justify-center'} ${className}`}
    >
      <div
        key={replayKey}
        onMouseMove={onMove}
        onMouseLeave={() => { setPointer(null); setHover(null) }}
        className={`glass pointer-events-auto relative flex ${flow} ${origin} ${vertical ? 'rounded-[28px] px-2 py-3' : 'rounded-[28px] px-3 py-2'} ${charging ? 'dock-charging' : ''}`}
        style={{
          gap,
          animation: dur ? `dock-in-${position} ${dur}ms cubic-bezier(.2,.9,.25,1) both` : undefined,
        }}
        role="toolbar"
        aria-label="Whirlpool dock demo"
      >
        {items.map((g, i) => {
          const s = sizeOf(i)
          const label = hover === g.id
          return (
            <Fragment key={g.id}>
            {i === sepAfter && (
              <span aria-hidden className="self-center bg-white/25" style={vertical ? { height: 1, width: size * 0.7 } : { width: 1, height: size * 0.75, marginBottom: 6 }} />
            )}
            <div className={`flex ${vertical ? 'flex-row' : 'flex-col'} items-center`}>
              <button
                type="button"
                aria-label={g.label}
                onMouseEnter={() => setHover(g.id)}
                onFocus={() => setHover(g.id)}
                onBlur={() => setHover(null)}
                onClick={() => { setBounce(g.id); window.setTimeout(() => setBounce(null), 900) }}
                className="relative grid place-items-center rounded-2xl outline-none focus-visible:ring-2 focus-visible:ring-[#5ff0e6]"
                style={{
                  width: s, height: s,
                  transition: `width ${ms}ms cubic-bezier(.2,.8,.2,1), height ${ms}ms cubic-bezier(.2,.8,.2,1)`,
                  animation: bounce === g.id ? 'icon-bounce .9s cubic-bezier(.3,.7,.4,1)' : undefined,
                }}
              >
                <Tile g={g} size={s} />
                {label && (
                  <span
                    className={`glass pointer-events-none absolute z-10 whitespace-nowrap rounded-lg px-2.5 py-1 text-[12px] font-medium text-white ${
                      position === 'bottom' ? 'bottom-full mb-3' : position === 'left' ? 'left-full ml-3' : 'right-full mr-3'
                    }`}
                  >
                    {g.label}
                  </span>
                )}
              </button>
              {g.running && (
                <span
                  aria-hidden
                  className={`rounded-full bg-[#5ff0e6] shadow-[0_0_8px_#5ff0e6] ${vertical ? (position === 'left' ? 'order-first mr-1' : 'ml-1') : 'mt-1'}`}
                  style={{ width: 4, height: 4 }}
                />
              )}
            </div>
            </Fragment>
          )
        })}
      </div>
    </div>
  )
}
