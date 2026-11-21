"""Draws the Whirlpool icon: a three-armed vortex in foam on a deep-sea squircle.

Everything is analytic (no raster assets). The master is rendered at 2x and
downsampled, so edges are crisp and the area outside the squircle is fully
transparent (no halo). Small sizes use a bolder variant so the arms still read
at 16-48 px.

Writes assets/brand/whirlpool-1024.png and assets/brand/whirlpool.ico.
Usage: python scripts/make_icon.py   (needs numpy and pillow)
"""

from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "assets" / "brand"

SS = 2
N = 1024 * SS
ICO_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
MARGIN = 72 / 1024
EXPONENT = 5.0

# Abyss -> tide -> lagoon, along the tile diagonal (0 = top-left).
RAMP = (
    (0.00, (22, 120, 132)),
    (0.35, (8, 78, 96)),
    (0.70, (5, 44, 60)),
    (1.00, (3, 24, 34)),
)
FOAM = np.array((0.78, 0.98, 0.95), np.float32)
WHITE = np.array((1.0, 1.0, 1.0), np.float32)


def grid():
    axis = (np.arange(N, dtype=np.float32) + 0.5) / N
    return np.meshgrid(axis, axis)


def coverage(distance):
    return np.clip(distance * N + 0.5, 0.0, 1.0).astype(np.float32)


def squircle(x, y, inset=0.0):
    half = 0.5 - MARGIN - inset
    u = np.abs(x - 0.5) / half
    v = np.abs(y - 0.5) / half
    radius = (u**EXPONENT + v**EXPONENT) ** (1 / EXPONENT)
    return coverage((1 - radius) * half)


def ramp(t):
    t = np.clip(t, 0.0, 1.0)
    out = np.zeros(t.shape + (3,), np.float32)
    for (t0, c0), (t1, c1) in zip(RAMP, RAMP[1:]):
        inside = (t >= t0) & (t <= t1)
        k = ((t - t0) / (t1 - t0))[inside][:, None]
        out[inside] = np.array(c0) * (1 - k) + np.array(c1) * k
    return out / 255.0


def over(dst, colour, alpha):
    a = alpha[..., None]
    dst[:] = dst * (1 - a) + np.asarray(colour, np.float32) * a


def smooth(edge0, edge1, v):
    t = np.clip((v - edge0) / (edge1 - edge0), 0, 1)
    return t * t * (3 - 2 * t)


def draw(boost: float) -> Image.Image:
    x, y = grid()
    tile = squircle(x, y)
    img = ramp((x * 0.7 + y * 0.6) / 1.3)
    img = img.astype(np.float32)
    # Light from the upper left, like sun on water.
    glow = np.clip(1 - np.hypot(x - 0.25, y - 0.2) / 0.8, 0, 1) ** 2
    over(img, (0.55, 0.95, 0.9), glow * 0.22)

    dx, dy = x - 0.5, y - 0.52
    r = np.hypot(dx, dy)
    theta = np.arctan2(dy, dx)
    # Log-spiral phase: two arms, tightening towards the eye.
    phase = 3 * (theta + 2.0 * np.log(np.maximum(r, 1e-4)))
    wave = 0.5 + 0.5 * np.cos(phase)
    # Arms taper to nothing at the eye and fade out toward the rim.
    width = 0.16 + 0.26 * smooth(0.04, 0.36, r) + boost
    arm = smooth(1 - width - 0.018, 1 - width + 0.018, wave)
    reach = smooth(0.375, 0.30, r) * smooth(0.012, 0.05, r)
    arm = arm * reach
    # Foam gets whiter toward the outside, like a breaking wave.
    tone = FOAM[None, None, :] * (1 - smooth(0.05, 0.36, r))[..., None] + WHITE * smooth(0.05, 0.36, r)[..., None]
    a = (arm * 0.96).astype(np.float32)
    over(img, tone, a)
    # Soft halo under the arms, for depth.
    halo = smooth(0.52, 0.8, np.cos(phase) * 0.5 + 0.5) * smooth(0.42, 0.2, r) * 0.10
    over(img, (0.4, 0.9, 0.9), halo.astype(np.float32) * (1 - a))
    # The eye.
    over(img, (1, 1, 1), coverage(0.022 + boost * 0.03 - r))

    ring = tile - squircle(x, y, inset=3 / 1024)
    over(img, (1, 1, 1), ring * (0.20 + 0.30 * (1 - y)))
    over(img, (1, 1, 1), np.clip(0.5 - y, 0, 0.5) * 0.10)

    rgba = np.dstack([np.clip(img, 0, 1), tile])
    rgba[..., :3] *= rgba[..., 3:4]
    return Image.fromarray((rgba * 255 + 0.5).astype(np.uint8))


def shrink(master, size):
    small = np.asarray(master.resize((size, size), Image.LANCZOS)).astype(np.float32) / 255
    alpha = np.clip(small[..., 3:4], 0, 1)
    colour = np.where(alpha > 0, small[..., :3] / np.maximum(alpha, 1e-6), 0)
    out = np.dstack([np.clip(colour, 0, 1), alpha])
    return Image.fromarray((out * 255 + 0.5).astype(np.uint8))


def main():
    fine, medium, bold = draw(0.0), draw(0.07), draw(0.14)
    shrink(fine, 1024).save(BRAND / "whirlpool-1024.png", optimize=True)
    pick = lambda s: bold if s <= 48 else (medium if s <= 128 else fine)
    frames = [shrink(pick(s), s) for s in ICO_SIZES]
    frames[-1].save(
        BRAND / "whirlpool.ico",
        sizes=[(s, s) for s in ICO_SIZES],
        append_images=frames[:-1],
        bitmap_format="png",
    )
    print("wrote whirlpool-1024.png and whirlpool.ico")


if __name__ == "__main__":
    main()
