import type { ReactNode, SVGProps } from "react";

type P = SVGProps<SVGSVGElement>;

function Svg({ children, ...p }: P & { children: ReactNode }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...p}
    >
      {children}
    </svg>
  );
}

export const Wifi = (p: P) => (
  <Svg {...p}>
    <path d="M2.5 9a15 15 0 0 1 19 0" />
    <path d="M5.5 12.5a10.5 10.5 0 0 1 13 0" />
    <path d="M8.7 16a6 6 0 0 1 6.6 0" />
    <circle cx="12" cy="19.2" r="1" fill="currentColor" />
  </Svg>
);
export const Bluetooth = (p: P) => (
  <Svg {...p}>
    <path d="m7 7 10 10-5 4V3l5 4L7 17" />
  </Svg>
);
export const Volume = (p: P) => (
  <Svg {...p}>
    <path d="M11 5 6 9H3v6h3l5 4V5Z" />
    <path d="M15.5 8.5a5 5 0 0 1 0 7" />
    <path d="M18.5 5.5a9 9 0 0 1 0 13" />
  </Svg>
);
export const Bolt = (p: P) => (
  <Svg {...p}>
    <path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z" fill="currentColor" stroke="none" />
  </Svg>
);
export const Battery = ({ level = 0.8, ...p }: P & { level?: number }) => (
  <svg viewBox="0 0 28 14" fill="none" aria-hidden="true" {...p}>
    <rect x="0.75" y="0.75" width="23" height="12.5" rx="3.5" stroke="currentColor" strokeOpacity="0.55" strokeWidth="1.5" />
    <rect x="2.6" y="2.6" width={Math.max(1, 19.3 * level)} height="8.8" rx="1.8" fill="currentColor" />
    <path d="M26 4.8v4.4" stroke="currentColor" strokeOpacity="0.55" strokeWidth="1.8" strokeLinecap="round" />
  </svg>
);
export const Search = (p: P) => (
  <Svg {...p}>
    <circle cx="11" cy="11" r="6.5" />
    <path d="m20 20-4.2-4.2" />
  </Svg>
);
export const Folder = (p: P) => (
  <Svg {...p}>
    <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5h3.6l2 2.2h7.4A2.5 2.5 0 0 1 21 9.7v7.8a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17.5v-10Z" />
  </Svg>
);
export const Globe = (p: P) => (
  <Svg {...p}>
    <circle cx="12" cy="12" r="9" />
    <path d="M3 12h18M12 3c2.7 2.6 3.9 5.6 3.9 9S14.700 18.400 12 21c-2.700-2.600-3.900-5.600-3.900-9S9.300 5.600 12 3Z" />
  </Svg>
);
export const Terminal = (p: P) => (
  <Svg {...p}>
    <path d="m5 8 4 4-4 4M12 17h7" />
  </Svg>
);
export const Music = (p: P) => (
  <Svg {...p}>
    <path d="M9 18V6l10-2v12" />
    <circle cx="6.5" cy="18" r="2.5" />
    <circle cx="16.500" cy="16" r="2.500" />
  </Svg>
);
export const Mail = (p: P) => (
  <Svg {...p}>
    <rect x="3" y="5" width="18" height="14" rx="2.500" />
    <path d="m3.500 7 8.500 6 8.500-6" />
  </Svg>
);
export const Gear = (p: P) => (
  <Svg {...p}>
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2.800v2.400M12 18.800v2.400M21.200 12h-2.400M5.200 12H2.800M18.500 5.500l-1.700 1.700M7.200 16.800l-1.700 1.700M18.500 18.500l-1.700-1.700M7.200 7.200 5.500 5.500" />
  </Svg>
);
export const Download = (p: P) => (
  <Svg {...p}>
    <path d="M12 4v11m0 0-4-4m4 4 4-4M5 20h14" />
  </Svg>
);
export const Trash = (p: P) => (
  <Svg {...p}>
    <path d="M4 7h16M10 11v6M14 11v6M6 7l1 12.500A1.500 1.500 0 0 0 8.500 21h7a1.500 1.500 0 0 0 1.500-1.500L18 7M9 7V4.500h6V7" />
  </Svg>
);
export const Github = (p: P) => (
  <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...p}>
    <path d="M12 .5a11.500 11.500 0 0 0-3.640 22.410c.580.110.790-.250.790-.560v-2c-3.200.700-3.880-1.360-3.880-1.360-.520-1.330-1.280-1.690-1.280-1.690-1.040-.710.080-.700.080-.700 1.150.080 1.760 1.180 1.760 1.180 1.030 1.760 2.700 1.250 3.360.960.100-.750.400-1.250.730-1.540-2.560-.290-5.250-1.280-5.250-5.700 0-1.260.450-2.290 1.180-3.100-.120-.290-.510-1.470.110-3.060 0 0 .970-.310 3.170 1.180a11 11 0 0 1 5.780 0c2.200-1.490 3.170-1.180 3.170-1.180.620 1.590.230 2.770.110 3.060.740.810 1.180 1.840 1.180 3.100 0 4.430-2.700 5.400-5.270 5.690.410.360.780 1.060.780 2.140v3.170c0 .310.210.680.800.560A11.500 11.500 0 0 0 12 .5Z" />
  </svg>
);
export const Check = (p: P) => (
  <Svg {...p}>
    <path d="m5 12.500 4.500 4.500L19 7.500" />
  </Svg>
);
export const Copy = (p: P) => (
  <Svg {...p}>
    <rect x="9" y="9" width="11" height="11" rx="2.500" />
    <path d="M5 15V6.500A1.500 1.500 0 0 1 6.500 5H15" />
  </Svg>
);
export const Shield = (p: P) => (
  <Svg {...p}>
    <path d="M12 3 4.500 6v5.500c0 4.500 3.100 8 7.500 9.500 4.400-1.500 7.500-5 7.500-9.500V6L12 3Z" />
    <path d="m9 12 2.200 2.200L15.500 10" />
  </Svg>
);
export const Arrow = (p: P) => (
  <Svg {...p}>
    <path d="M5 12h14m-5-5 5 5-5 5" />
  </Svg>
);
export const Play = (p: P) => (
  <Svg {...p}>
    <path d="M7 5v14l12-7L7 5Z" fill="currentColor" stroke="none" />
  </Svg>
);
export const Grid = (p: P) => (
  <Svg {...p}>
    <rect x="4" y="4" width="6.500" height="6.500" rx="1.500" />
    <rect x="13.500" y="4" width="6.500" height="6.500" rx="1.500" />
    <rect x="4" y="13.500" width="6.500" height="6.500" rx="1.500" />
    <rect x="13.500" y="13.500" width="6.500" height="6.500" rx="1.500" />
  </Svg>
);
export const Palette = (p: P) => (
  <Svg {...p}>
    <path d="M12 3a9 9 0 1 0 0 18c1.400 0 2-1 1.500-2.100-.500-1.100.200-2.400 1.500-2.400H17a4 4 0 0 0 4-4C21 6.700 17 3 12 3Z" />
    <circle cx="7.500" cy="11" r="1" fill="currentColor" />
    <circle cx="10.500" cy="7.500" r="1" fill="currentColor" />
    <circle cx="15.500" cy="8" r="1" fill="currentColor" />
  </Svg>
);
export const Windows = (p: P) => (
  <Svg {...p}>
    <rect x="3" y="4.500" width="18" height="15" rx="2.500" />
    <path d="M3 9h18" />
  </Svg>
);
export const Cpu = (p: P) => (
  <Svg {...p}>
    <rect x="6" y="6" width="12" height="12" rx="2" />
    <path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3" />
  </Svg>
);
export const Bar = (p: P) => (
  <Svg {...p}>
    <rect x="3" y="4.500" width="18" height="5" rx="1.500" />
    <path d="M3 14h18M3 18h12" />
  </Svg>
);
