// Mirrors the Rust models in src-tauri/src/library/models.rs and settings.rs.
// Keep both sides in sync when changing a field.

export type PlatformId =
  | "steam" | "epic" | "rockstar" | "gog" | "ea" | "ubisoft" | "battlenet" | "xbox" | "custom";

export interface Account {
  platform: PlatformId;
  /** Platform-side id, e.g. SteamID64. */
  id: string;
  displayName: string;
  /** Login name the switcher uses (never a password). */
  loginName?: string;
  /** Currently signed in on the platform client. */
  active: boolean;
}

export interface Game {
  /** Stable id across platforms, e.g. "steam:271590". */
  key: string;
  title: string;
  platform: PlatformId;
  /** Platform-side app id. */
  appId: string;
  /** Account that owns the game (local files). */
  ownerAccountId?: string;
  /** Account the launcher switches to before launching (after rules). */
  launchAccountId?: string;
  /** Protected by the Call of Duty safety rule. */
  guarded: boolean;
  installed: boolean;
  installDir?: string;
  sizeBytes?: number;
  playtimeMinutes?: number;
  /** Unix seconds. */
  lastPlayed?: number;
  /** Microsoft Store id (Xbox games). */
  storeId?: string;
}

export interface PlatformStatus {
  id: PlatformId;
  name: string;
  detected: boolean;
  installPath?: string;
  gameCount: number;
}

export interface Art {
  cover?: string;
  hero?: string;
}

export interface Library {
  platforms: PlatformStatus[];
  accounts: Account[];
  games: Game[];
  /** Game key -> art for non-Steam games. */
  art: Record<string, Art>;
}

export interface Theme {
  accent: string;
  accent2: string;
  /** Window background. */
  bg: string;
  /** Glow strength 0–200 (%). */
  glow: number;
  /** Glass opacity 0–200 (%). */
  glass: number;
  /** Glass blur px. */
  blur: number;
}

export interface StorageReport {
  drives: { root: string; total: number; free: number; games: number }[];
  games: { key: string; title: string; platform: PlatformId; bytes: number; dir: string }[];
  modsBytes: number;
  mediaBytes: number;
}

export interface Settings {
  /** Game key -> forced account id. */
  pinnedAccounts: Record<string, string>;
  /** Steam accounts banned in Call of Duty. */
  codBannedAccounts: string[];
  /** Main Steam account ("" = auto: the one owning the most games). */
  primaryAccount: string;
  favorites: string[];
  hidden: string[];
  minimizeOnPlay: boolean;
  theme: Theme;
  steamgriddbKey: string;
  steamApiKey: string;
  /** Anthropic key: Claude writes descriptions in Saudi Arabic. */
  anthropicKey: string;
  nexusKey: string;
  /** "glass" | "amoled" */
  mode: string;
  profile: Profile;
  /** Game key -> custom poster paths (several = slideshow). */
  customCovers: Record<string, string[]>;
}

export interface Profile {
  name: string;
  bio: string;
  /** Saved image paths (GIF/WebP animations allowed). */
  avatar: string;
  header: string;
}

export interface Deal { store: string; trust: "official" | "authorized"; price: string; retail: string; savings: number; url: string }

/** A mod the user installed by hand (detected automatically). */
export interface ExternalMod { id: string; name: string; kind: "loader" | "mod" | "graphics"; paths: string[]; enabled: boolean }

export interface Details {
  steamAppId?: number;
  name: string;
  description: string;
  background?: string;
  screenshots: string[];
  genres: string[];
  developers: string[];
  publishers: string[];
  releaseDate: string;
  metacritic?: number;
  price?: string;
  currentPlayers?: number;
  awards: string[];
  goty: boolean;
  deals: Deal[];
}

export interface StoreItem { appId: number; name: string; image: string; price?: string; discount: number }

export interface ModInfo {
  modId: number;
  domain: string;
  name: string;
  summary: string;
  picture?: string;
  thumbnail?: string;
  endorsements: number;
  downloads: number;
  author: string;
  version: string;
  updatedAt: string;
  description: string;
  images: string[];
  url: string;
}

export interface InstalledMod {
  id: string;
  gameKey: string;
  domain: string;
  modId: number;
  fileId: number;
  name: string;
  version: string;
  picture?: string;
  enabled: boolean;
  target: string;
  files: string[];
  installedAt: number;
}

export interface ModSupport { domain?: string; nexusName?: string; blocked?: string; warning?: string }
