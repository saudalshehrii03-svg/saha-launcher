import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Art, Details, InstalledMod, Library, ModInfo, ModSupport, Settings, StorageReport, StoreItem } from "./types";

type Prog = { key: string; message: string };

// Thin typed wrappers around Tauri commands (src-tauri/src/commands.rs).
export const api = {
  appInfo: () => invoke<{ name: string; version: string }>("app_info"),
  scanLibrary: () => invoke<Library>("scan_library"),
  fetchArt: () => invoke<Record<string, Art>>("fetch_art"),
  syncOwned: () => invoke<number>("sync_owned"),
  gameDetails: (title: string, steamAppId?: number) => invoke<Details>("game_details", { title, steamAppId: steamAppId ?? null }),
  storeFeatured: () => invoke<Record<string, StoreItem[]>>("store_featured"),
  storeSearch: (term: string) => invoke<StoreItem[]>("store_search", { term }),
  launchGame: (key: string) => invoke<void>("launch_game", { key }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (value: Settings) => invoke<void>("save_settings", { value }),
  saveMedia: (name: string, data: string) => invoke<string>("save_media", { name, data }),
  modSupport: (title: string) => invoke<ModSupport>("mod_support", { title }),
  modsBrowse: (domain: string, term: string, sort: string) => invoke<ModInfo[]>("mods_browse", { domain, term, sort }),
  modPage: (domain: string, modId: number) => invoke<ModInfo>("mod_page", { domain, modId }),
  modsInstalled: () => invoke<InstalledMod[]>("mods_installed"),
  modInstall: (gameKey: string, opts: { link?: string; domain?: string; modId?: number }) =>
    invoke<InstalledMod>("mod_install", { gameKey, link: opts.link ?? null, domain: opts.domain ?? null, modId: opts.modId ?? null }),
  modSetEnabled: (id: string, enabled: boolean) => invoke<InstalledMod>("mod_set_enabled", { id, enabled }),
  modRemove: (id: string) => invoke<void>("mod_remove", { id }),
  openExternal: (url: string) => invoke<void>("open_external", { url }),
  storageReport: () => invoke<StorageReport>("storage_report"),
  openFolder: (path: string) => invoke<void>("open_folder", { path }),
  onLaunchProgress: (cb: (p: Prog) => void) => listen<Prog>("launch-progress", (e) => cb(e.payload)),
  onModProgress: (cb: (p: Prog) => void) => listen<Prog>("mod-progress", (e) => cb(e.payload)),
  onGameState: (cb: (p: { key: string; running: boolean }) => void) =>
    listen<{ key: string; running: boolean }>("game-state", (e) => cb(e.payload)),
};

/** Local saved image -> URL the webview can show. */
export const mediaUrl = (path: string) => (path ? convertFileSrc(path) : "");

/** Read a picked file and store it; returns the saved path. */
export async function uploadImage(file: File): Promise<string> {
  const data = await new Promise<string>((res, rej) => {
    const r = new FileReader();
    r.onload = () => res(String(r.result));
    r.onerror = rej;
    r.readAsDataURL(file);
  });
  return api.saveMedia(file.name, data);
}

/** Nexus domain -> library game key, remembered from game pages (for nxm:// links). */
export const nexusGames = {
  get(domain: string): string | undefined {
    try {
      return JSON.parse(localStorage.getItem("saha.nexus") || "{}")[domain];
    } catch {
      return undefined;
    }
  },
  set(domain: string, key: string) {
    try {
      const m = JSON.parse(localStorage.getItem("saha.nexus") || "{}");
      m[domain] = key;
      localStorage.setItem("saha.nexus", JSON.stringify(m));
    } catch {
      /* storage unavailable */
    }
  },
};
