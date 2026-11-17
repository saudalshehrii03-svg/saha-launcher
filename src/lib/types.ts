// Mirrors the Rust models in src-tauri/src/library/models.rs.
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
}

export interface Game {
  /** Stable id across platforms, e.g. "steam:271590". */
  key: string;
  title: string;
  platform: PlatformId;
  /** Platform-side app id. */
  appId: string;
  /** Account that owns the game. Launch must switch to it first. */
  ownerAccountId?: string;
  installed: boolean;
  installDir?: string;
  sizeBytes?: number;
  playtimeMinutes?: number;
  lastPlayed?: string;
}

export interface PlatformStatus {
  id: PlatformId;
  name: string;
  detected: boolean;
  installPath?: string;
  gameCount: number;
}
