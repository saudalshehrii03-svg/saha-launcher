import { useEffect, useState } from "react";
import { PLATFORMS, artUrls } from "../lib/platforms";
import type { Art as ArtUrls, Game, PlatformId } from "../lib/types";
import { mediaUrl, uploadImage } from "../lib/api";

const PATHS: Record<string, string> = {
  play: "M7 4.5v15l13-7.5z",
  search: "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-4-4",
  grid: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
  layers: "M12 3 3 8l9 5 9-5zM3 13l9 5 9-5",
  user: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 21a8 8 0 0 1 16 0",
  gear: "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
  sync: "M20 11a8 8 0 0 0-14.9-3M4 4v4h4M4 13a8 8 0 0 0 14.9 3M20 20v-4h-4",
  shield: "M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6z",
  puzzle: "M10 3a2 2 0 0 1 4 0v2h4v4h-2a2 2 0 0 0 0 4h2v4h-4v-2a2 2 0 0 0-4 0v2H6v-4h2a2 2 0 0 0 0-4H6V5h4z",
  x: "M6 6l12 12M18 6 6 18",
  check: "M5 12l5 5 9-10",
  heart: "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
  eyeOff: "M3 3l18 18M10.6 6.1A9.6 9.6 0 0 1 12 6c5 0 9 6 9 6a15 15 0 0 1-2.6 3.2M6.6 6.6C4.3 8.1 3 12 3 12s4 6 9 6c1.4 0 2.7-.4 3.9-1",
  eye: "M3 12s4-6 9-6 9 6 9 6-4 6-9 6-9-6-9-6zM12 9.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z",
  dice: "M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM8.5 8.5h.01M15.5 15.5h.01M12 12h.01",
  store: "M4 7h16l-1.5 11a2 2 0 0 1-2 1.7H7.5a2 2 0 0 1-2-1.7zM9 7V5a3 3 0 0 1 6 0v2",
  trophy: "M8 4h8v5a4 4 0 0 1-8 0zM8 6H5a3 3 0 0 0 3 4M16 6h3a3 3 0 0 1-3 4M12 13v4M8 21h8M10 17h4",
  people: "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM2.5 20a6.5 6.5 0 0 1 13 0M16 4.5a3.5 3.5 0 0 1 0 6.8M18 14a6.5 6.5 0 0 1 3.5 6",
  tag: "M3 12V4a1 1 0 0 1 1-1h8l9 9-9 9zM7.5 7.5h.01",
  back: "M15 5l-7 7 7 7",
  download: "M12 4v11M7 10l5 5 5-5M5 20h14",
  trash: "M5 7h14M10 7V4h4v3M7 7l1 13h8l1-13",
  power: "M12 3v8M6.3 7a8 8 0 1 0 11.4 0",
  external: "M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5",
  image: "M4 5h16v14H4zM4 15l5-5 4 4 3-3 4 4M15 9h.01",
  sparkle: "M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z",
  palette:"M12 3a9 9 0 0 0 0 18c1 0 1.5-.8 1.5-1.5 0-1-.8-1.4-.8-2.3 0-.9.7-1.7 1.7-1.7H17a4 4 0 0 0 4-4c0-4.7-4-8.5-9-8.5zM7.5 11h.01M10 7h.01M15 7h.01",
};

export function Icon({ name, size = 20 }: { name: string; size?: number }) {
  const fill = name === "play" || name === "heart-on";
  if (name === "heart-on") name = "heart";
  return (
    <svg className="i" width={size} height={size} viewBox="0 0 24 24" fill={fill ? "currentColor" : "none"}>
      <path d={PATHS[name]} />
    </svg>
  );
}

export function Badge({ platform, full }: { platform: PlatformId; full?: boolean }) {
  const p = PLATFORMS[platform];
  return (
    <span className="badge" style={{ background: p.bg, color: p.fg }}>
      {full ? p.name : p.name.split(" ")[0]}
    </span>
  );
}

/** Image with URL fallbacks; renders `fallback` when all fail or none exist. */
/**
 * Game art. `custom` (user posters, GIF/WebP animations work) wins over official art;
 * several custom posters rotate every second with a crossfade.
 */
export function Art({ game, kind, art, custom, className, children, onClick }: {
  game: Game; kind: "cover" | "hero"; art?: ArtUrls; custom?: string[]; className: string; children?: React.ReactNode; onClick?: () => void;
}) {
  const urls = artUrls(game, kind, art);
  const [i, setI] = useState(0);
  const first = urls[0];
  useEffect(() => setI(0), [first]); // art arrived later (cover lookup)
  const url = urls[i];
  const mine = (custom ?? []).map(mediaUrl);
  const [slide, setSlide] = useState(0);
  useEffect(() => {
    if (mine.length < 2) return;
    const t = setInterval(() => setSlide((s) => s + 1), 1000);
    return () => clearInterval(t);
  }, [mine.length]);
  const hue = 340 + ([...game.title].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 40, 7) - 20);
  return (
    <div
      className={className}
      onClick={onClick}
      style={{ background: `linear-gradient(160deg, hsl(${hue} 10% 15%), hsl(${hue} 8% 6%) 70%, #000)` }}
    >
      {mine.length > 0 ? (
        mine.map((m, n) => <img key={m} src={m} alt="" className={`slide ${n === slide % mine.length ? "on" : ""}`} draggable={false} />)
      ) : (
        url && <img src={url} alt="" onError={() => setI((n) => n + 1)} draggable={false} />
      )}
      {!url && mine.length === 0 && kind === "cover" && <span className="art-title">{game.title}</span>}
      {children}
    </div>
  );
}

/** File picker button for images (GIF/WebP animations allowed). */
export function ImagePick({ label, multiple, onPick, className = "btn2 glass" }: { label: React.ReactNode; multiple?: boolean; onPick: (paths: string[]) => void; className?: string }) {
  return (
    <label className={className} style={{ cursor: "pointer" }}>
      {label}
      <input
        type="file"
        accept="image/png,image/jpeg,image/gif,image/webp,image/avif"
        multiple={multiple}
        hidden
        onChange={async (e) => {
          const files = [...(e.target.files ?? [])];
          e.target.value = "";
          if (files.length) onPick(await Promise.all(files.map(uploadImage)));
        }}
      />
    </label>
  );
}
