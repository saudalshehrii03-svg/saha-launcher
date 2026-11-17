import { useCallback, useEffect, useState } from "react";
import { api } from "./api";
import type { Library, PlatformId, Settings, Theme } from "./types";

export const DEFAULT_THEME: Theme = { accent: "#ff2a4d", accent2: "#8a0f24", bg: "#030304", glow: 60, glass: 100, blur: 28 };

export type LaunchState = { key: string; message: string; status: "running" | "done" | "error" } | null;
export type Filter = PlatformId | "all" | "favorites" | "hidden";

const EMPTY: Library = { platforms: [], accounts: [], games: [], art: {} };
export const DEFAULT_SETTINGS: Settings = {
  pinnedAccounts: {},
  codBannedAccounts: [],
  favorites: [],
  hidden: [],
  minimizeOnPlay: true,
  theme: DEFAULT_THEME,
  steamgriddbKey: "",
  steamApiKey: "",
  nexusKey: "",
  mode: "glass",
  profile: { name: "", bio: "", avatar: "", header: "" },
  customCovers: {},
};

/** App-wide state: library, settings, the current launch and running games. */
export function useLibrary() {
  const [lib, setLib] = useState<Library>(EMPTY);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [scanning, setScanning] = useState(true);
  const [error, setError] = useState<string>("");
  const [launch, setLaunch] = useState<LaunchState>(null);
  const [running, setRunning] = useState<string[]>([]);
  const [filter, setFilter] = useState<Filter>("all");
  /** game key -> [enabled, total] mods (hand-installed + Saha). */
  const [modCounts, setModCounts] = useState<Record<string, [number, number]>>({});
  const refreshModCounts = useCallback(() => {
    api.modsCounts().then(setModCounts).catch(() => {});
  }, []);

  const fetchArt = useCallback(() => {
    api.fetchArt().then((art) => setLib((l) => ({ ...l, art }))).catch(() => {});
  }, []);

  const rescan = useCallback(async () => {
    setScanning(true);
    try {
      setLib(await api.scanLibrary());
      setError("");
      fetchArt(); // covers in the background
      refreshModCounts(); // detect installed mods for every game
    } catch (e) {
      setError(String(e));
    } finally {
      setScanning(false);
    }
  }, [fetchArt, refreshModCounts]);

  useEffect(() => {
    api
      .getSettings()
      .then((s) => {
        setSettings({ ...DEFAULT_SETTINGS, ...s, theme: { ...DEFAULT_THEME, ...s.theme } });
        // Owned games refresh once per start (needs the Steam Web API key).
        if (s.steamApiKey) api.syncOwned().then(() => rescan()).catch(() => {});
      })
      .catch(() => {});
    rescan();
    const subs = [
      api.onLaunchProgress((p) => setLaunch({ ...p, status: "running" })),
      api.onModProgress((p) => setLaunch({ ...p, status: "running" })),
      api.onGameState((p) => {
        setRunning((r) => (p.running ? [...r.filter((k) => k !== p.key), p.key] : r.filter((k) => k !== p.key)));
        if (!p.running) rescan(); // pick up the new playtime
      }),
    ];
    return () => subs.forEach((s) => s.then((f) => f()).catch(() => {}));
  }, [rescan]);

  /** Save settings; rescan only when it changes which account a game uses. */
  const saveSettings = useCallback(
    async (s: Settings, opts: { rescan?: boolean } = {}) => {
      setSettings(s);
      await api.saveSettings(s);
      if (opts.rescan) rescan();
    },
    [rescan],
  );

  const toggleIn = useCallback(
    (list: "favorites" | "hidden", key: string) => {
      const cur = settings[list];
      saveSettings({ ...settings, [list]: cur.includes(key) ? cur.filter((k) => k !== key) : [...cur, key] });
    },
    [settings, saveSettings],
  );

  const play = useCallback(async (key: string) => {
    setLaunch({ key, message: "جاري التجهيز…", status: "running" });
    try {
      await api.launchGame(key);
      setLaunch({ key, message: "تم التشغيل", status: "done" });
      setTimeout(() => setLaunch((l) => (l?.key === key && l.status === "done" ? null : l)), 4000);
    } catch (e) {
      setLaunch({ key, message: String(e), status: "error" });
    }
  }, []);

  /** Pull every owned Steam game (Web API key), then rescan. */
  const syncOwned = useCallback(async () => {
    await api.syncOwned();
    await rescan();
  }, [rescan]);

  /** Show a message in the bottom toast (mods, store actions). */
  const notify = useCallback((key: string, message: string, status: "running" | "done" | "error") => {
    setLaunch({ key, message, status });
    if (status === "done") setTimeout(() => setLaunch((l) => (l?.message === message ? null : l)), 4000);
  }, []);

  return {
    lib, syncOwned, notify, modCounts, refreshModCounts, settings, scanning, error, launch, running, filter,
    setFilter, rescan, saveSettings, toggleIn, play, fetchArt,
    dismissLaunch: () => setLaunch(null),
  };
}

export type Store = ReturnType<typeof useLibrary>;
