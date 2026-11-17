import { invoke } from "@tauri-apps/api/core";
import type { Game, PlatformStatus } from "./types";

// Thin typed wrappers around Tauri commands (src-tauri/src/commands.rs).
export const api = {
  appInfo: () => invoke<{ name: string; version: string }>("app_info"),
  listPlatforms: () => invoke<PlatformStatus[]>("list_platforms"),
  scanLibrary: () => invoke<Game[]>("scan_library"),
};
