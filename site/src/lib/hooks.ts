import { useEffect, useRef, useState } from "react";

export const REPO = "prashantchataut/whirlpool";
export const REPO_URL = `https://github.com/${REPO}`;
export const LATEST_EXE = `${REPO_URL}/releases/latest/download/whirlpool.exe`;
export const INSTALL_LINE = `irm https://raw.githubusercontent.com/${REPO}/main/scripts/install.ps1 | iex`;

export type ReleaseAsset = { name: string; size: number; url: string };
export type Release = { tag: string; published: string; page: string; assets: ReleaseAsset[] };

/** Reads the real latest release from GitHub; the page still works if it can't. */
export function useLatestRelease() {
  const [release, setRelease] = useState<Release | null>(null);
  const [state, setState] = useState<"loading" | "ready" | "none">("loading");
  useEffect(() => {
    const ctrl = new AbortController();
    fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { signal: ctrl.signal })
      .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
      .then((j) => {
        setRelease({
          tag: j.tag_name,
          published: j.published_at,
          page: j.html_url,
          assets: (j.assets ?? []).map((a: any) => ({
            name: a.name,
            size: a.size,
            url: a.browser_download_url,
          })),
        });
        setState("ready");
      })
      .catch(() => setState("none"));
    return () => ctrl.abort();
  }, []);
  return { release, state };
}

export function formatSize(bytes: number) {
  return bytes > 1024 * 1024
    ? `${(bytes / 1024 / 1024).toFixed(1)} MB`
    : `${Math.max(1, Math.round(bytes / 1024))} KB`;
}

/** Adds `.in` to `.reveal` elements as they enter the viewport. */
export function useReveal() {
  useEffect(() => {
    const els = Array.from(document.querySelectorAll<HTMLElement>(".reveal"));
    const io = new IntersectionObserver(
      (entries) =>
        entries.forEach((e) => {
          if (e.isIntersecting) {
            e.target.classList.add("in");
            io.unobserve(e.target);
          }
        }),
      { threshold: 0.12, rootMargin: "0px 0px -6% 0px" },
    );
    els.forEach((el) => io.observe(el));
    return () => io.disconnect();
  }, []);
}

/** 0 at the top of the page, 1 at the bottom. */
export function useScrollProgress() {
  const [p, setP] = useState(0);
  const [y, setY] = useState(0);
  const raf = useRef(0);
  useEffect(() => {
    const read = () => {
      const max = document.documentElement.scrollHeight - window.innerHeight;
      setP(max > 0 ? Math.min(1, Math.max(0, window.scrollY / max)) : 0);
      setY(window.scrollY);
    };
    const on = () => {
      cancelAnimationFrame(raf.current);
      raf.current = requestAnimationFrame(read);
    };
    read();
    window.addEventListener("scroll", on, { passive: true });
    window.addEventListener("resize", on);
    return () => {
      window.removeEventListener("scroll", on);
      window.removeEventListener("resize", on);
      cancelAnimationFrame(raf.current);
    };
  }, []);
  return { progress: p, y };
}
