// One place for the release the site points at. `scripts/site-release.mjs` rewrites SUMS from SHA256SUMS.txt.
export const VERSION = "4.2.0-rc9";
export const TAG = `v${VERSION}`;
const BASE = `https://github.com/prashantchataut/whirlpool/releases/download/${TAG}`;

export const REPO = "https://github.com/prashantchataut/whirlpool";
export const RELEASE_URL = `${REPO}/releases/tag/${TAG}`;
export const SUMS_URL = `${BASE}/SHA256SUMS.txt`;

export const FILES = [
  { arch: "x64", kind: "Installer", name: "Whirlpool-Setup.exe", url: `${BASE}/Whirlpool-Setup.exe` },
  { arch: "ARM64", kind: "Installer", name: "Whirlpool-Setup-arm64.exe", url: `${BASE}/Whirlpool-Setup-arm64.exe` },
  { arch: "x64", kind: "Portable", name: "Whirlpool-x64.zip", url: `${BASE}/Whirlpool-x64.zip` },
  { arch: "ARM64", kind: "Portable", name: "Whirlpool-arm64.zip", url: `${BASE}/Whirlpool-arm64.zip` },
] as const;

/** SHA-256 per file, copied from SHA256SUMS.txt of the release. */
export const SUMS: Record<string, string> = {};

export const CREDIT = "Curated 4K wallpapers sourced from 4kwallpapers.com. Free for personal desktop use.";
export const CREDIT_URL = "https://4kwallpapers.com/";
