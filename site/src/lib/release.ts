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
export const SUMS: Record<string, string> = {
  "Whirlpool-Setup.exe": "88e27d24340489bb87e86a36319db89b63e583c57477395a60cd87dda8661d74",
  "Whirlpool-Setup-arm64.exe": "b0c2d907e28913555b5cc994f01deee363e6526444f9aad0bbae2921a040ac07",
  "Whirlpool-x64.zip": "4ee50c12f03789fc4b4bebdba7d1266b8c1d1e7d25ec3f17b1b187c3afe8d803",
  "Whirlpool-arm64.zip": "915a7e3030a8cbda143fd8d61449004acefc5acf0abb38b5030507900ba65ab4",
};

export const CREDIT = "Curated 4K wallpapers sourced from 4kwallpapers.com. Free for personal desktop use.";
export const CREDIT_URL = "https://4kwallpapers.com/";
