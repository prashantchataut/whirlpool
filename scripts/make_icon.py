"""Draws the Whirlpool icon: three tapering spiral arms winding into a point of
light, on a deep-water squircle.

Everything is analytic: arms are defined as distance fields, edges get exact
anti-aliasing, and the image is rendered at 2x and downsampled, so the area
outside the squircle is fully transparent (no halo).

Writes assets/brand/whirlpool-1024.png and assets/brand/whirlpool.ico.

Usage: pip install pillow numpy && python scripts/make_icon.py
"""

from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "assets" / "brand"

SS = 2
OUT = 1024
N = OUT * SS
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)

MARGIN = 72 / 1024
EXPONENT = 5.0
ARMS = 3
WIND = 0.62        # how tightly the arms wind (radians of turn per e-fold of radius)
R_MAX = 0.355      # outer radius of the arms, in unit coordinates
R_MIN = 0.018

DEEP = np.array((3, 24, 34), np.float32)       # abyss
MID = np.array((8, 78, 94), np.float32)        # water
FOAM = np.array((148, 247, 232), np.float32)   # foam
TEAL = np.array((14, 165, 168), np.float32)    # current


def coverage(signed: np.ndarray) -> np.ndarray:
    """Signed distance in unit coordinates -> pixel coverage."""
    return np.clip(signed * N + 0.5, 0.0, 1.0).astype(np.float32)


def main() -> None:
    axis = (np.arange(N, dtype=np.float32) + 0.5) / N
    x, y = np.meshgrid(axis, axis)

    # Tile.
    half = 0.5 - MARGIN
    u, v = np.abs(x - 0.5) / half, np.abs(y - 0.5) / half
    radius = (u**EXPONENT + v**EXPONENT) ** (1 / EXPONENT)
    tile = coverage((1 - radius) * half)

    # Background: light falls from the top-left, the bottom sinks into the abyss.
    t = np.clip((x * 0.35 + y * 0.65 - 0.1) / 0.8, 0, 1)[..., None]
    rgb = MID * (1 - t) + DEEP * t
    dx, dy = x - 0.5, y - 0.5
    r = np.hypot(dx, dy)
    glow = np.exp(-((r / 0.30) ** 2))[..., None]
    rgb = rgb + glow * np.array((10, 60, 66), np.float32)

    # Spiral arms.
    theta = np.arctan2(dy, dx)
    rr = np.clip(r, 1e-6, None)
    arm_cov = np.zeros_like(r)
    arm_color = np.zeros_like(rgb)
    for i in range(ARMS):
        phase = theta - np.log(rr / R_MAX) / WIND - i * (2 * np.pi / ARMS)
        phase = (phase + np.pi) % (2 * np.pi) - np.pi
        along = np.clip((R_MAX - rr) / (R_MAX - R_MIN), 0, 1)  # 0 outside, 1 centre
        width = 0.058 * (0.22 + 0.78 * (1 - along) ** 0.9)      # thick outside, thin inside
        dist = np.abs(phase) * rr                                # ~ perpendicular distance
        c = coverage(width / 2 - dist)
        c *= (rr < R_MAX).astype(np.float32) * (rr > R_MIN).astype(np.float32)
        # Soft fade at both ends.
        c *= np.clip((R_MAX - rr) / 0.03, 0, 1) ** 0.7 * np.clip((rr - R_MIN) / 0.02, 0, 1)
        shade = np.clip(along, 0, 1)[..., None]
        col = TEAL * (1 - shade) + FOAM * shade
        arm_color = np.where(c[..., None] > arm_cov[..., None], col, arm_color)
        arm_cov = np.maximum(arm_cov, c)
    rgb = rgb * (1 - arm_cov[..., None]) + arm_color * arm_cov[..., None]

    # Eye of the whirlpool.
    eye = np.clip(1 - r / 0.03, 0, 1) ** 1.5
    rgb = rgb + eye[..., None] * (FOAM - rgb) * 0.9
    halo = np.exp(-((r / 0.075) ** 2)) * 0.22
    rgb = rgb + halo[..., None] * (FOAM - rgb)

    # Rim light along the top edge of the tile.
    rim = np.clip(1 - np.abs(radius - 0.985) / 0.012, 0, 1) * np.clip(1 - y * 1.15, 0, 1)
    rgb = rgb + rim[..., None] * (FOAM - rgb) * 0.35

    out = np.dstack([np.clip(rgb, 0, 255), tile * 255]).astype(np.uint8)
    image = Image.fromarray(out, "RGBA").resize((OUT, OUT), Image.LANCZOS)
    BRAND.mkdir(parents=True, exist_ok=True)
    image.save(BRAND / "whirlpool-1024.png")
    image.save(BRAND / "whirlpool.ico", sizes=[(s, s) for s in ICO_SIZES])
    print("wrote", BRAND / "whirlpool-1024.png", "and whirlpool.ico")


if __name__ == "__main__":
    main()
