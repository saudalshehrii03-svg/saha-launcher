import { useCallback, useEffect, useState } from "react";
import { onOpenUrl, getCurrent } from "@tauri-apps/plugin-deep-link";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

/** F11 toggles full screen, Esc leaves it. */
export async function toggleFullscreen(force?: boolean) {
  const w = getCurrentWindow();
  await w.setFullscreen(force ?? !(await w.isFullscreen()));
}
import Sidebar, { type PageId } from "./components/Sidebar";
import LaunchToast from "./components/LaunchToast";
import UpdateBanner from "./components/UpdateBanner";
import Library from "./pages/Library";
import Platforms from "./pages/Platforms";
import Mods from "./pages/Mods";
import Settings from "./pages/Settings";
import GamePage from "./pages/GamePage";
import StorePage from "./pages/Store";
import Profile from "./pages/Profile";
import { useLibrary } from "./lib/useLibrary";
import { applyTheme } from "./lib/platforms";
import { api, nexusGames } from "./lib/api";

/** What the main area shows. "game" = game page (owned or from the store). */
export type View =
  | { page: Exclude<PageId, "game"> }
  | { page: "game"; key?: string; appId?: number; title: string; tab?: "info" | "mods" | "installed" };
export type Nav = { go: (v: View) => void; back: () => void; forward: () => void };

/** Liquid-glass light that follows the pointer over any glass surface. */
function useGlassLight() {
  useEffect(() => {
    const move = (e: PointerEvent) => {
      const el = (e.target as HTMLElement | null)?.closest<HTMLElement>(".glass, .poster, .chip, .nav-item, .play, .theme");
      if (!el) return;
      const r = el.getBoundingClientRect();
      el.style.setProperty("--mx", `${e.clientX - r.left}px`);
      el.style.setProperty("--my", `${e.clientY - r.top}px`);
    };
    window.addEventListener("pointermove", move, { passive: true });
    return () => window.removeEventListener("pointermove", move);
  }, []);
}

export default function App() {
  // Browser-like history: back/forward with the mouse side buttons or Alt+←/→.
  const [hist, setHist] = useState<{ stack: View[]; at: number }>({ stack: [{ page: "library" }], at: 0 });
  const view = hist.stack[hist.at];
  const nav: Nav = {
    go: (v) =>
      setHist((h) => {
        if (JSON.stringify(h.stack[h.at]) === JSON.stringify(v)) return h;
        const stack = [...h.stack.slice(0, h.at + 1), v].slice(-30);
        return { stack, at: stack.length - 1 };
      }),
    back: () => setHist((h) => ({ ...h, at: Math.max(0, h.at - 1) })),
    forward: () => setHist((h) => ({ ...h, at: Math.min(h.stack.length - 1, h.at + 1) })),
  };
  useEffect(() => {
    // Mouse button 3 = back (thumb, near you), 4 = forward.
    const block = (e: MouseEvent) => {
      if (e.button === 3 || e.button === 4) e.preventDefault();
    };
    const up = (e: MouseEvent) => {
      if (e.button === 3) setHist((h) => ({ ...h, at: Math.max(0, h.at - 1) }));
      if (e.button === 4) setHist((h) => ({ ...h, at: Math.min(h.stack.length - 1, h.at + 1) }));
      block(e);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "F11") {
        e.preventDefault();
        toggleFullscreen().catch(() => {});
        return;
      }
      if (e.key === "Escape") toggleFullscreen(false).catch(() => {});
      if (!e.altKey) return;
      // RTL: Alt+→ goes back, Alt+← forward (matches the reading direction)
      if (e.key === "ArrowRight") setHist((h) => ({ ...h, at: Math.max(0, h.at - 1) }));
      if (e.key === "ArrowLeft") setHist((h) => ({ ...h, at: Math.min(h.stack.length - 1, h.at + 1) }));
    };
    window.addEventListener("mousedown", block);
    window.addEventListener("mouseup", up);
    window.addEventListener("keydown", key);
    return () => {
      window.removeEventListener("mousedown", block);
      window.removeEventListener("mouseup", up);
      window.removeEventListener("keydown", key);
    };
  }, []);
  const store = useLibrary();
  const theme = store.settings.theme;
  useEffect(() => applyTheme(theme), [theme]);
  useEffect(() => {
    document.documentElement.dataset.mode = store.settings.mode || "glass";
  }, [store.settings.mode]);
  useGlassLight();

  // nxm:// links from Nexus "Mod Manager Download" -> install into the matching game.
  const { notify, lib } = store;
  const handleLinks = useCallback(
    (urls: string[] | null) => {
      for (const url of urls ?? []) {
        if (!url.startsWith("nxm://")) continue;
        const domain = url.slice(6).split("/")[0].toLowerCase();
        const key = nexusGames.get(domain);
        if (!key) {
          notify("mods", "افتح صفحة مودات اللعبة في ساحة مرة وحدة، ثم اضغط Mod Manager Download من جديد.", "error");
          continue;
        }
        notify(key, "استلمت المود من Nexus…", "running");
        api
          .modInstall(key, { link: url })
          .then((m) => notify(key, `تم تثبيت «${m.name}»`, "done"))
          .catch((e) => notify(key, String(e), "error"));
      }
    },
    [notify],
  );
  useEffect(() => {
    getCurrent().then(handleLinks).catch(() => {});
    const un = onOpenUrl(handleLinks);
    // nxm:// caught inside the in-app Nexus window
    const un2 = listen<string>("nxm-link", (e) => handleLinks([e.payload]));
    return () => {
      un.then((f) => f()).catch(() => {});
      un2.then((f) => f()).catch(() => {});
    };
  }, [handleLinks]);

  const page = view.page;
  return (
    <div className="shell">
      <div className="ambient" aria-hidden />
      <Sidebar page={page} onNavigate={(p) => nav.go({ page: p } as View)} store={store} />
      <main className="main" key={page === "game" ? `game:${view.key ?? view.appId}` : page}>
        {page === "library" && <Library store={store} nav={nav} />}
        {page === "store" && <StorePage store={store} nav={nav} />}
        {page === "profile" && <Profile store={store} nav={nav} />}
        {page === "platforms" && <Platforms store={store} />}
        {page === "mods" && <Mods store={store} nav={nav} />}
        {page === "settings" && <Settings store={store} />}
        {view.page === "game" && <GamePage store={store} nav={nav} view={view} game={lib.games.find((g) => g.key === view.key || (view.appId && g.platform === "steam" && g.appId === String(view.appId)))} />}
      </main>
      <LaunchToast store={store} />
      <UpdateBanner />
    </div>
  );
}
