"""Draws the Whirlpool mark: a luminous three-arm vortex on a glass squircle.

Everything is analytic (no raster assets), so the logo is reproducible:

    python scripts/make_icon.py        # needs numpy and pillow

Writes
  assets/brand/whirlpool-1024.png   master raster
  assets/brand/whirlpool.ico        16..256 px (small sizes use a bolder variant)
  assets/brand/whirlpool.svg        vector twin of the mark (same maths)
  assets/brand/whirlpool-mono.svg   single-colour mark for docs
  site/public/*                     favicon + touch icon copies (if site/public exists)

The shape language: three tapered arms follow a logarithmic spiral
(theta = K * ln r + phase). Arms are shaded like glass tubes, lit from the
upper left, and bloom into a bright eye. The tile is a superellipse (n = 5),
the same curve as the app icons the dock sits beside.
"""

from __future__ import annotations

import math
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "assets" / "brand"

N = 2048  # supersampled master
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
MARGIN = 0.07
EXP = 5.0
K = 1.75  # spiral tightness
ARMS = 3
R_OUT = 0.74
R_IN = 0.07

# Tide ramp, rim -> eye.
RING = (
    (0.00, (24, 160, 196)),
    (0.45, (74, 222, 232)),
    (0.80, (178, 250, 250)),
    (1.00, (255, 255, 255)),
)


def smoothstep(a: float, b: float, x):
    t = np.clip((np.asarray(x, dtype=np.float32) - a) / (b - a), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def ramp(stops, t: np.ndarray) -> np.ndarray:
    out = np.zeros(t.shape + (3,), dtype=np.float32)
    out[:] = stops[0][1]
    for (t0, c0), (t1, c1) in zip(stops, stops[1:]):
        m = (t >= t0) & (t <= t1)
        k = ((t - t0) / (t1 - t0))[m][:, None]
        out[m] = np.array(c0) * (1 - k) + np.array(c1) * k
    out[t > stops[-1][0]] = stops[-1][1]
    return out


def half_width(r, bold: bool):
    w0 = 0.115 if bold else 0.085
    # Swell out of the eye, taper toward the rim.
    return w0 * smoothstep(0.04, 0.22, r) * (1.15 - 0.75 * smoothstep(0.2, R_OUT, r))


def render(size: int = N, bold: bool = False) -> Image.Image:
    ys, xs = np.mgrid[0:size, 0:size].astype(np.float32)
    u = (xs + 0.5) / size * 2 - 1
    v = (ys + 0.5) / size * 2 - 1
    aa = 2.2 / size

    # Squircle tile.
    s = 1 - MARGIN
    d_tile = (np.abs(u) ** EXP + np.abs(v) ** EXP) ** (1 / EXP) - s
    tile_a = np.clip(0.5 - d_tile / aa, 0, 1)

    # Deep-sea background, lit from the upper left.
    diag = np.clip((u + v) * 0.25 + 0.5, 0, 1)[..., None]
    top = np.array((12, 96, 120), dtype=np.float32)
    mid = np.array((5, 44, 66), dtype=np.float32)
    bot = np.array((2, 14, 30), dtype=np.float32)
    bg = np.where(
        diag < 0.5,
        top * (1 - diag * 2) + mid * (diag * 2),
        mid * (1 - (diag - 0.5) * 2) + bot * ((diag - 0.5) * 2),
    )
    r = np.sqrt(u * u + v * v)
    bg = bg + np.array((18, 170, 190), dtype=np.float32) * (0.28 * np.exp(-((r / 0.5) ** 2)))[..., None]
    rgb = bg.copy()

    # Vortex arms.
    theta = np.arctan2(v, u)
    arms_mask = np.zeros_like(r)
    tube = np.zeros_like(r)
    spec = np.zeros_like(r)
    safe_r = np.maximum(r, 1e-3)
    hw = half_width(r, bold)
    for i in range(ARMS):
        t = theta - K * np.log(safe_r) - 2 * math.pi * i / ARMS
        t = (t + math.pi) % (2 * math.pi) - math.pi
        arc = t * r
        m = np.clip((hw - np.abs(arc)) / aa, 0, 1)
        m *= 1 - smoothstep(R_OUT - 0.1, R_OUT + 0.02, r)
        m *= smoothstep(R_IN * 0.4, R_IN * 1.4, r)
        sn = np.clip(arc / np.maximum(hw, 1e-4), -1, 1)
        h = np.sqrt(np.clip(1 - sn * sn, 0, 1))
        better = m > arms_mask
        tube = np.where(better, h * (1 - 0.18 * sn), tube)
        spec = np.where(better, np.clip(1 - np.abs(sn + 0.42) / 0.16, 0, 1) * m, spec)
        arms_mask = np.maximum(arms_mask, m)

    depth = 1 - smoothstep(R_IN, R_OUT, r)  # 1 at the eye, 0 at the rim
    arm_rgb = ramp(RING, depth)
    shade = 0.5 + 0.5 * tube
    arm_rgb = arm_rgb * shade[..., None] + 255 * (0.55 * spec)[..., None]

    # Bloom behind the arms.
    glow_src = Image.fromarray((np.clip(arms_mask * (0.4 + 0.6 * depth), 0, 1) * 255).astype(np.uint8))
    glow = np.asarray(glow_src.filter(ImageFilter.GaussianBlur(size * 0.022)), dtype=np.float32) / 255
    glow2 = np.asarray(glow_src.filter(ImageFilter.GaussianBlur(size * 0.06)), dtype=np.float32) / 255
    rgb = rgb + np.array((40, 220, 235), dtype=np.float32) * (0.55 * glow + 0.5 * glow2)[..., None]
    rgb = rgb * (1 - arms_mask[..., None]) + arm_rgb * arms_mask[..., None]

    # The eye: a bright droplet.
    eye_r = 0.08 if bold else 0.062
    eye = np.clip((eye_r - r) / aa, 0, 1)
    eye_col = ramp(((0, (120, 240, 245)), (1, (255, 255, 255))), np.clip(1 - r / eye_r, 0, 1))
    rgb = rgb * (1 - eye[..., None]) + eye_col * eye[..., None]
    halo = np.exp(-((r / 0.13) ** 2))
    rgb = rgb + np.array((150, 255, 255), dtype=np.float32) * (0.45 * halo)[..., None]

    # Glass: a soft highlight over the upper half and a bright upper-left rim.
    gloss = np.clip(1 - (v + 0.18 * u * u + 0.18) / 0.95, 0, 1) ** 2.2
    gloss = gloss * smoothstep(-0.95, -0.55, v) * (1 - smoothstep(-0.1, 0.35, v))
    rgb = rgb + 255 * (0.12 * gloss)[..., None]
    rim = np.clip(1 - np.abs(d_tile + 0.012) / 0.012, 0, 1)
    rim_light = np.clip(0.5 - 0.45 * (u + v), 0.08, 1.0)
    rgb = rgb + 255 * (0.42 * rim * rim_light)[..., None]
    # Darken the lower edge for depth.
    rgb = rgb * (1 - 0.32 * (smoothstep(0.55, 1.0, v) * smoothstep(-0.2, 0.0, d_tile + 0.15)))[..., None]

    rgb = np.clip(rgb, 0, 255)
    out = np.dstack([rgb, tile_a * 255]).astype(np.uint8)
    return Image.fromarray(out, "RGBA")


# ---------------------------------------------------------------- vector twin
def arm_polygon(i: int, bold: bool = False, steps: int = 90) -> str:
    left, right = [], []
    for rr in np.linspace(R_IN * 0.9, R_OUT - 0.03, steps):
        th = K * math.log(rr) + 2 * math.pi * i / ARMS
        taper = 1 - float(smoothstep(R_OUT - 0.12, R_OUT, rr)) * 0.9
        w = float(half_width(rr, bold)) * taper
        # Tangent of r(th) = exp((th - phase)/K): dr/dth = r/K.
        tx = math.cos(th) / K - math.sin(th)
        ty = math.sin(th) / K + math.cos(th)
        length = math.hypot(tx, ty)
        nx, ny = -ty / length, tx / length
        px, py = rr * math.cos(th), rr * math.sin(th)
        left.append((px + nx * w, py + ny * w))
        right.append((px - nx * w, py - ny * w))
    pts = left + right[::-1]
    return " ".join(f"{512 + x * 512:.1f},{512 + y * 512:.1f}" for x, y in pts)


def squircle_path(half: float, n: float = EXP, pts: int = 160) -> str:
    out = []
    for k in range(pts):
        a = 2 * math.pi * k / pts
        c, si = math.cos(a), math.sin(a)
        x = math.copysign(abs(c) ** (2 / n), c) * half
        y = math.copysign(abs(si) ** (2 / n), si) * half
        out.append(f"{512 + x:.1f},{512 + y:.1f}")
    return "M" + " L".join(out) + " Z"


def svg(mono: bool = False) -> str:
    fill = "currentColor" if mono else "url(#arm)"
    arms = "\n      ".join(f'<polygon points="{arm_polygon(i)}" fill="{fill}"/>' for i in range(ARMS))
    if mono:
        return (
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" role="img" aria-label="Whirlpool">\n'
            f'  {arms}\n  <circle cx="512" cy="512" r="64" fill="currentColor"/>\n</svg>\n'
        )
    half = (1 - MARGIN) * 512
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" role="img" aria-label="Whirlpool">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#0c6078"/><stop offset=".5" stop-color="#052c42"/><stop offset="1" stop-color="#020e1e"/>
    </linearGradient>
    <radialGradient id="core" cx=".5" cy=".5" r=".5">
      <stop offset="0" stop-color="#12aabe" stop-opacity=".55"/><stop offset="1" stop-color="#12aabe" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="arm" cx="512" cy="512" r="380" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#ffffff"/><stop offset=".25" stop-color="#b2fafa"/>
      <stop offset=".6" stop-color="#4adee8"/><stop offset="1" stop-color="#18a0c4"/>
    </radialGradient>
    <linearGradient id="gloss" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity=".22"/><stop offset="1" stop-color="#fff" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="rim" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity=".55"/><stop offset=".5" stop-color="#fff" stop-opacity=".05"/><stop offset="1" stop-color="#fff" stop-opacity="0"/>
    </linearGradient>
    <filter id="bloom" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="16"/></filter>
    <clipPath id="tile"><path d="{squircle_path(half)}"/></clipPath>
  </defs>
  <path d="{squircle_path(half)}" fill="url(#bg)"/>
  <g clip-path="url(#tile)">
    <circle cx="512" cy="512" r="260" fill="url(#core)"/>
    <g filter="url(#bloom)" opacity=".75">
      {arms}
    </g>
    {arms}
    <circle cx="512" cy="512" r="64" fill="#fff"/>
    <circle cx="512" cy="512" r="130" fill="url(#core)"/>
    <rect x="0" y="0" width="1024" height="520" fill="url(#gloss)" opacity=".6"/>
  </g>
  <path d="{squircle_path(half - 6)}" fill="none" stroke="url(#rim)" stroke-width="7"/>
</svg>
"""


def main() -> None:
    BRAND.mkdir(parents=True, exist_ok=True)
    master = render(N, bold=False).resize((1024, 1024), Image.LANCZOS)
    master.save(BRAND / "whirlpool-1024.png")

    frames = []
    for px in ICO_SIZES:
        bold = px <= 48
        big = render(px * 4 if px < 128 else N, bold=bold)
        frames.append(big.resize((px, px), Image.LANCZOS))
    frames[-1].save(
        BRAND / "whirlpool.ico",
        format="ICO",
        sizes=[(f.width, f.height) for f in frames],
        append_images=frames[:-1],
    )
    (BRAND / "whirlpool.svg").write_text(svg(False), encoding="utf-8")
    (BRAND / "whirlpool-mono.svg").write_text(svg(True), encoding="utf-8")

    public = ROOT / "site" / "public"
    if public.exists():
        master.resize((512, 512), Image.LANCZOS).save(public / "logo-512.png")
        master.resize((180, 180), Image.LANCZOS).save(public / "apple-touch-icon.png")
        (public / "favicon.svg").write_text(svg(False), encoding="utf-8")
    print("wrote brand assets")


if __name__ == "__main__":
    main()
