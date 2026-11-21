import { useEffect, useState } from 'react'
import logo from '../assets/logo.png'

type Props = {
  /** Tint strength over the glass, in percent (config: menuBarOpacity). */
  opacity?: number
  showDate?: boolean
  /** null follows the system, like the app's `menuBar24h`. */
  hour24?: boolean | null
  charging?: boolean
  compact?: boolean
  className?: string
}

const Icon = ({ d, label }: { d: string; label: string }) => (
  <svg viewBox="0 0 24 24" role="img" aria-label={label} className="h-[15px] w-[15px]" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" strokeLinejoin="round">
    <path d={d} />
  </svg>
)

export default function MenuBar({ opacity = 50, showDate = true, hour24 = null, charging = false, compact = false, className = '' }: Props) {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    // One wake-up per minute, like the app.
    const id = window.setInterval(() => setNow(new Date()), 30_000)
    return () => window.clearInterval(id)
  }, [])
  const time = now.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit', hour12: hour24 === null ? undefined : !hour24 })
  const date = now.toLocaleDateString([], { weekday: 'short', day: 'numeric', month: 'short' })
  const a = Math.min(1, Math.max(0.1, opacity / 100))
  return (
    <div
      className={`flex items-center justify-between px-3 text-white ${compact ? 'h-6 text-[11px]' : 'h-7 text-[12.5px]'} ${className}`}
      style={{
        background: `rgba(5, 26, 38, ${a * 0.9})`,
        backdropFilter: 'blur(22px) saturate(160%)',
        WebkitBackdropFilter: 'blur(22px) saturate(160%)',
        boxShadow: 'inset 0 -1px 0 rgba(255,255,255,.12)',
      }}
    >
      <div className="flex items-center gap-4">
        <img src={logo} alt="" className="h-4 w-4" />
        <span className="font-semibold">Whirlpool</span>
        {!compact && <span className="hidden text-white/70 sm:inline">File</span>}
        {!compact && <span className="hidden text-white/70 sm:inline">View</span>}
        {!compact && <span className="hidden text-white/70 md:inline">Window</span>}
      </div>
      <div className="flex items-center gap-3.5">
        <Icon label="Network" d="M2.5 9a14 14 0 0 1 19 0M5.5 12.5a9.5 9.5 0 0 1 13 0M8.6 16a5 5 0 0 1 6.8 0M12 19.4h.01" />
        <Icon label="Volume" d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5zM15.5 9a4 4 0 0 1 0 6M18 6.5a8 8 0 0 1 0 11" />
        <span className={`flex items-center gap-1.5 tabular-nums ${charging ? 'text-[#ffc857]' : ''}`} aria-label={charging ? 'Battery charging' : 'Battery'}>
          <svg viewBox="0 0 28 14" className="h-3.5 w-7" fill="none" stroke="currentColor" strokeWidth="1.4">
            <rect x="1" y="1" width="23" height="12" rx="3.5" />
            <rect x="25.2" y="4.5" width="1.8" height="5" rx=".8" fill="currentColor" stroke="none" />
            <rect x="3" y="3" width={charging ? 14 : 17} height="8" rx="2" fill="currentColor" stroke="none" />
            {charging && <path d="m13.5 2 -4 6h3l-1 4 4-6h-3z" fill="#04141f" stroke="none" transform="translate(2 0)" />}
          </svg>
          {!compact && <span>{charging ? '64%' : '82%'}</span>}
        </span>
        <span className="tabular-nums">
          {showDate && <span className="mr-2 text-white/75">{date}</span>}
          {time}
        </span>
      </div>
    </div>
  )
}
