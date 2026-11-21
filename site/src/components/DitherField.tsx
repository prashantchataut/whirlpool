import { useEffect, useRef } from 'react'

// Ordered (Bayer 8x8) dither of a slowly turning spiral field. It draws at a
// fraction of the display resolution, pauses off-screen and under
// prefers-reduced-motion, so the effect costs almost nothing.
const BAYER = (() => {
  const m = [
    [0, 2],
    [3, 1],
  ]
  let size = 2
  let cur = m
  while (size < 8) {
    const next: number[][] = Array.from({ length: size * 2 }, () => Array(size * 2).fill(0))
    for (let y = 0; y < size; y++)
      for (let x = 0; x < size; x++) {
        const v = cur[y][x] * 4
        next[y][x] = v
        next[y][x + size] = v + 2
        next[y + size][x] = v + 3
        next[y + size][x + size] = v + 1
      }
    cur = next
    size *= 2
  }
  return cur.map((r) => r.map((v) => (v + 0.5) / 64))
})()

type Props = {
  className?: string
  /** Pixel size of one dither cell on screen. */
  cell?: number
  /** Foreground colour of lit cells, as [r, g, b]. */
  tint?: [number, number, number]
  /** 0..1, how much of the field is lit. */
  density?: number
  center?: [number, number]
}

export default function DitherField({
  className,
  cell = 4,
  tint = [95, 240, 230],
  density = 0.55,
  center = [0.68, 0.45],
}: Props) {
  const ref = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = ref.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    let raf = 0
    let visible = true
    let last = 0
    let w = 0
    let h = 0
    let img: ImageData

    const resize = () => {
      const r = canvas.getBoundingClientRect()
      w = Math.max(8, Math.ceil(r.width / cell))
      h = Math.max(8, Math.ceil(r.height / cell))
      canvas.width = w
      canvas.height = h
      img = ctx.createImageData(w, h)
    }

    const draw = (t: number) => {
      const aspect = w / h
      const cx = center[0] * aspect
      const cy = center[1]
      const data = img.data
      for (let y = 0; y < h; y++) {
        const ny = y / h - cy
        for (let x = 0; x < w; x++) {
          const nx = (x / w) * aspect - cx
          const d = Math.sqrt(nx * nx + ny * ny) + 0.02
          const a = Math.atan2(ny, nx)
          const arm = 0.5 + 0.5 * Math.sin(a * 3 - 7.5 * Math.log(d) - t * 0.45)
          const fall = Math.exp(-d * 1.55)
          const ring = 0.5 + 0.5 * Math.sin(d * 26 - t * 0.9)
          const v = (arm * 0.78 + ring * 0.22) * fall * 1.5 * density
          const lit = v > BAYER[y & 7][x & 7]
          const i = (y * w + x) * 4
          data[i] = tint[0]
          data[i + 1] = tint[1]
          data[i + 2] = tint[2]
          data[i + 3] = lit ? Math.min(255, 70 + v * 210) : 0
        }
      }
      ctx.putImageData(img, 0, 0)
    }

    const loop = (now: number) => {
      raf = requestAnimationFrame(loop)
      if (!visible || now - last < 1000 / 20) return
      last = now
      draw(now / 1000)
    }

    resize()
    draw(2)
    const ro = new ResizeObserver(() => {
      resize()
      draw(performance.now() / 1000)
    })
    ro.observe(canvas)
    const io = new IntersectionObserver(([e]) => (visible = e.isIntersecting))
    io.observe(canvas)
    if (!reduce) raf = requestAnimationFrame(loop)
    return () => {
      cancelAnimationFrame(raf)
      ro.disconnect()
      io.disconnect()
    }
  }, [cell, tint, density, center])

  return (
    <canvas
      ref={ref}
      aria-hidden="true"
      className={className}
      style={{ imageRendering: 'pixelated', width: '100%', height: '100%' }}
    />
  )
}
