import type { Art, Game, PlatformId } from "./types";

export const PLATFORMS: Record<PlatformId, { name: string; short: string; bg: string; fg: string }> = {
  steam: { name: "Steam", short: "S", bg: "#1b2838", fg: "#66c0f4" },
  epic: { name: "Epic Games", short: "E", bg: "#2a2a2a", fg: "#ffffff" },
  rockstar: { name: "Rockstar", short: "R", bg: "#fcaf17", fg: "#000000" },
  gog: { name: "GOG", short: "G", bg: "#86328a", fg: "#ffffff" },
  ea: { name: "EA app", short: "EA", bg: "#ff4747", fg: "#ffffff" },
  ubisoft: { name: "Ubisoft", short: "U", bg: "#0070ff", fg: "#ffffff" },
  battlenet: { name: "Battle.net", short: "B", bg: "#148eff", fg: "#ffffff" },
  xbox: { name: "Xbox", short: "X", bg: "#107c10", fg: "#ffffff" },
  custom: { name: "مخصص", short: "+", bg: "rgba(255,255,255,.1)", fg: "#ffffff" },
};

/** Candidate image URLs, tried in order. Empty = use the generated placeholder. */
export function artUrls(g: Game, kind: "cover" | "hero", art?: Art): string[] {
  if (g.platform !== "steam") {
    const url = kind === "cover" ? art?.cover : art?.hero ?? art?.cover;
    return url ? [url] : [];
  }
  const file = kind === "cover" ? "library_600x900.jpg" : "library_hero.jpg";
  return [
    `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${g.appId}/${file}`,
    `https://cdn.akamai.steamstatic.com/steam/apps/${g.appId}/${file}`,
  ];
}

/** Color presets for Settings > Appearance. */
export const THEMES: { name: string; accent: string; accent2: string }[] = [
  { name: "أحمر ملكي", accent: "#ff2a4d", accent2: "#8a0f24" },
  { name: "قرمزي", accent: "#ff4d6d", accent2: "#3a0612" },
  { name: "تركواز", accent: "#00e0bd", accent2: "#7c5cff" },
  { name: "بنفسجي", accent: "#b18cff", accent2: "#ff5fa2" },
  { name: "ذهبي", accent: "#ffc04d", accent2: "#ff6a3d" },
  { name: "أزرق", accent: "#4da3ff", accent2: "#00e0bd" },
  { name: "أخضر", accent: "#5ef08a", accent2: "#00b4d8" },
];

/** Apply a theme to CSS variables (text on accent buttons picks black/white by contrast). */
export function applyTheme(t: { accent: string; accent2: string }) {
  const s = document.documentElement.style;
  s.setProperty("--accent", t.accent);
  s.setProperty("--accent-2", t.accent2);
  const hex = t.accent.replace("#", "");
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  s.setProperty("--on-accent", lum > 0.45 ? "#06120f" : "#ffffff");
}

const nf = new Intl.NumberFormat("ar-SA");
export const num = (n: number) => nf.format(n);

export function hours(min?: number) {
  if (!min) return "";
  return min < 60 ? `${num(min)} د` : `${num(Math.round(min / 60))} س`;
}

export function ago(unix?: number) {
  if (!unix) return "لم تُلعب بعد";
  const days = Math.floor((Date.now() / 1000 - unix) / 86400);
  if (days <= 0) return "اليوم";
  if (days === 1) return "أمس";
  if (days < 7) return `قبل ${num(days)} أيام`;
  if (days < 30) return `قبل ${num(Math.floor(days / 7))} أسابيع`;
  if (days < 365) return `قبل ${num(Math.floor(days / 30))} أشهر`;
  return "قبل أكثر من سنة";
}

export function gb(bytes?: number) {
  return bytes ? `${num(Math.round(bytes / 1e9))} GB` : "";
}
