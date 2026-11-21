// A TypeScript port of crates/whirlpool-core/src/wallpaper.rs, so the site draws
// the very wallpapers the app draws: same styles, same maths, same palettes.
import type { Preset } from './presets'

type Rgb = [number, number, number]

const hex = (c: string): Rgb => [
  parseInt(c.slice(1, 3), 16),
  parseInt(c.slice(3, 5), 16),
  parseInt(c.slice(5, 7), 16),
]

function hash(x: number, y: number, seed: number): number {
  let h =
    (Math.imul(x | 0, 0x27d4eb2d) ^ Math.imul(y | 0, 0x165667b1) ^ Math.imul(seed | 0, 0x9e3779b1)) >>> 0
  h ^= h >>> 15
  h = Math.imul(h, 0x85ebca6b) >>> 0
  h ^= h >>> 13
  h = Math.imul(h, 0xc2b2ae35) >>> 0
  h ^= h >>> 16
  return ((h >>> 0) & 0xffffff) / 16777215
}
const smooth = (t: number) => t * t * (3 - 2 * t)
const clamp = (x: number, a = 0, b = 1) => Math.min(b, Math.max(a, x))
const smoothstep = (a: number, b: number, x: number) => smooth(clamp((x - a) / (b - a)))

function vnoise(x: number, y: number, seed: number): number {
  const x0 = Math.floor(x)
  const y0 = Math.floor(y)
  const fx = smooth(x - x0)
  const fy = smooth(y - y0)
  const a = hash(x0, y0, seed)
  const b = hash(x0 + 1, y0, seed)
  const c = hash(x0, y0 + 1, seed)
  const d = hash(x0 + 1, y0 + 1, seed)
  const top = a + (b - a) * fx
  return top + (c + (d - c) * fx - top) * fy
}

function fbm(x: number, y: number, seed: number, octaves: number): number {
  let sum = 0
  let amp = 0.5
  let norm = 0
  let freq = 1
  for (let o = 0; o < octaves; o++) {
    sum += amp * vnoise(x * freq, y * freq, seed + o * 101)
    norm += amp
    amp *= 0.5
    freq *= 2
  }
  return sum / norm
}

const mix = (a: Rgb, b: Rgb, t: number): Rgb => {
  t = clamp(t)
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
const add = (a: Rgb, b: Rgb, k: number): Rgb => [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k]

function mesh(p: Preset, cols: Rgb[], x: number, y: number, aspect: number): Rgb {
  const wx = x + 0.2 * (fbm(x * 1.3, y * 1.3, p.seed, 3) - 0.5)
  const wy = y + 0.2 * (fbm(x * 1.3 + 7, y * 1.3 + 3, p.seed + 1, 3) - 0.5)
  let sum: Rgb = [0, 0, 0]
  let weight = 0
  cols.forEach((c, i) => {
    const ax = hash(i, 0, p.seed) * aspect
    const ay = hash(i, 1, p.seed)
    const d2 = (wx - ax) ** 2 + (wy - ay) ** 2
    const w = 1 / (d2 + 0.035) ** 2
    sum = add(sum, c, w)
    weight += w
  })
  return [sum[0] / weight, sum[1] / weight, sum[2] / weight]
}

function horizon(p: Preset, cols: Rgb[], x: number, y: number, aspect: number): Rgb {
  const [top, low, glow, far, near] = cols
  let col = mix(top, low, smoothstep(0, 0.7, y))
  const sx = (0.25 + 0.5 * hash(1, 1, p.seed)) * aspect
  const dx = x - sx
  const dy = y - 0.46
  const d2 = dx * dx + dy * dy
  col = add(col, glow, 0.85 * Math.exp(-d2 / 0.06))
  col = mix(col, glow, smoothstep(0.0016, 0.0009, d2))
  const layers = 5
  for (let i = 0; i < layers; i++) {
    const t = i / (layers - 1)
    const k = 1.5 + 0.7 * i
    const base = 0.5 + 0.1 * i
    const ridge =
      base +
      0.05 * Math.sin(x * k + p.seed + i * 1.7) +
      0.025 * Math.sin(x * (4 + i) + i * 2.3) +
      0.07 * (fbm(x * (1.2 + 0.3 * i), i * 3.1, p.seed, 3) - 0.5)
    if (y > ridge) {
      const body = mix(far, near, t)
      const haze = (1 - t) * 0.6 * (1 - smoothstep(ridge, ridge + 0.18, y))
      const sky = mix(top, low, smoothstep(0, 0.7, ridge))
      col = mix(body, sky, haze + (1 - t) * 0.12)
    }
  }
  return col
}

function vortex(cols: Rgb[], x: number, y: number, aspect: number): Rgb {
  const [outer, center, arms, eye, spare] = cols
  const dx = x - aspect * 0.5
  const dy = y - 0.5
  const d = Math.sqrt(dx * dx + dy * dy)
  const angle = Math.atan2(dy, dx)
  let col = mix(center, outer, smoothstep(0, 0.95, d))
  const phase = angle * 3 - 8 * Math.log(d + 0.03)
  const band = smoothstep(0.35, 1, Math.sin(phase))
  col = add(col, arms, 0.7 * band * Math.exp(-d * 2.4))
  const weave = smoothstep(0.7, 1, Math.sin(angle * 5 + 11 * Math.log(d + 0.03) + 1.3))
  col = add(col, spare, 0.25 * weave * Math.exp(-d * 3.4))
  col = add(col, arms, 0.05 * (0.5 + 0.5 * Math.sin(d * 90)) * Math.exp(-d * 3))
  col = add(col, eye, 1.1 * Math.exp((-d * d) / 0.004))
  col = add(col, arms, 0.5 * Math.exp((-d * d) / 0.03))
  return col
}

/** Draws a preset to RGBA pixels. Same look as the Windows app, any size. */
export function render(p: Preset, width: number, height: number): ImageData {
  const cols = p.colors.map(hex)
  const aspect = width / height
  const img = new ImageData(width, height)
  for (let py = 0; py < height; py++) {
    const y = (py + 0.5) / height
    for (let px = 0; px < width; px++) {
      const x = ((px + 0.5) / width) * aspect
      let c =
        p.style === 'mesh'
          ? mesh(p, cols, x, y, aspect)
          : p.style === 'horizon'
            ? horizon(p, cols, x, y, aspect)
            : vortex(cols, x, y, aspect)
      const vx = x / aspect - 0.5
      const vy = y - 0.5
      const v = 1 - 0.22 * smoothstep(0.25, 0.95, Math.sqrt(vx * vx + vy * vy) * 1.4)
      const grain = (hash(px, py, 0xd17e) - 0.5) * 1.8
      const i = (py * width + px) * 4
      img.data[i] = c[0] * v + grain
      img.data[i + 1] = c[1] * v + grain
      img.data[i + 2] = c[2] * v + grain
      img.data[i + 3] = 255
    }
  }
  return img
}

/** A dominant colour for tinting UI around a wallpaper. */
export const accentOf = (p: Preset): string => p.colors[2]
