import { useEffect, useRef } from "react";

/** 4x4 Bayer matrix, normalised to 0..1. */
const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5].map((v) => (v + 0.5) / 16);

const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

export type DitherVariant = "whirl" | "drift" | "rise" | "scan";

function field(variant: DitherVariant, x: number, y: number, aspect: number, t: number): number {
  const nx = (x - 0.5) * aspect;
  const ny = y - 0.5;
  switch (variant) {
    case "whirl": {
      const r = Math.hypot(nx, ny);
      const a = Math.atan2(ny, nx);
      const arms = 0.5 + 0.5 * Math.sin(a * 3 + 1.1 / (r + 0.1) - t * 0.55);
      const shaped = arms * arms * (3 - 2 * arms);
      const falloff = smooth(0.78, 0.05, r);
      const core = smooth(0.12, 0.0, r) * 0.9;
      return Math.min(1, shaped * falloff * 0.95 + core);
    }
    case "drift": {
      const v = 0.5 + 0.5 * Math.sin(nx * 4.2 + t * 0.4) * Math.sin(ny * 5.5 - t * 0.3 + nx * 1.5);
      return v * 0.8;
    }
    case "rise": {
      const wave = 0.5 + 0.5 * Math.sin(nx * 7 + t * 0.6 + ny * 3);
      return Math.pow(y, 2.2) * (0.55 + 0.45 * wave);
    }
    case "scan": {
      const band = 0.5 + 0.5 * Math.sin(y * 28 - t * 1.2);
      return (0.25 + 0.5 * band) * smooth(0, 0.5, x) * smooth(1, 0.6, x);
    }
  }
}

type Props = {
  variant?: DitherVariant;
  /** Size of one dither cell in CSS pixels. */
  cell?: number;
  /** `r,g,b` of the lit cells. */
  color?: string;
  fps?: number;
  speed?: number;
  className?: string;
  opacity?: number;
};

/**
 * Ordered-dither field on a low-resolution canvas, scaled up without smoothing.
 * Draws only while on screen; with reduced motion it draws one still frame.
 */
export function Dither({
  variant = "whirl",
  cell = 5,
  color = "62,230,204",
  fps = 18,
  speed = 1,
  className = "",
  opacity = 1,
}: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const [r, g, b] = color.split(",").map((n) => Number(n.trim()));
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let w = 0;
    let h = 0;
    let image: ImageData | null = null;
    let raf = 0;
    let last = 0;
    let visible = true;
    const t0 = performance.now();

    const resize = () => {
      const box = canvas.getBoundingClientRect();
      w = Math.max(8, Math.ceil(box.width / cell));
      h = Math.max(8, Math.ceil(box.height / cell));
      canvas.width = w;
      canvas.height = h;
      image = ctx.createImageData(w, h);
    };

    const draw = (now: number) => {
      if (!image) return;
      const t = ((now - t0) / 1000) * speed;
      const aspect = w / h;
      const data = image.data;
      for (let j = 0; j < h; j++) {
        for (let i = 0; i < w; i++) {
          const v = field(variant, i / w, j / h, aspect, t);
          const on = v > BAYER[(j & 3) * 4 + (i & 3)];
          const k = (j * w + i) * 4;
          data[k] = r;
          data[k + 1] = g;
          data[k + 2] = b;
          data[k + 3] = on ? 255 : 0;
        }
      }
      ctx.putImageData(image, 0, 0);
    };

    const loop = (now: number) => {
      raf = requestAnimationFrame(loop);
      if (!visible || now - last < 1000 / fps) return;
      last = now;
      draw(now);
    };

    resize();
    draw(performance.now());
    const observer = new ResizeObserver(() => {
      resize();
      draw(performance.now());
    });
    observer.observe(canvas);
    const io = new IntersectionObserver(([entry]) => {
      visible = entry.isIntersecting;
    });
    io.observe(canvas);
    if (!reduce) raf = requestAnimationFrame(loop);

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      io.disconnect();
    };
  }, [variant, cell, color, fps, speed]);

  return (
    <canvas
      ref={ref}
      aria-hidden="true"
      className={`pixel-mask block h-full w-full ${className}`}
      style={{ imageRendering: "pixelated", opacity }}
    />
  );
}
