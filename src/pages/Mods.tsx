import { useCallback, useEffect, useState } from "react";
import { Art, Icon } from "../components/ui";
import { api } from "../lib/api";
import { num } from "../lib/platforms";
import type { InstalledMod } from "../lib/types";
import type { Store } from "../lib/useLibrary";
import type { Nav } from "../App";

/** All installed mods across games + shortcuts to each game's mod tabs. */
export default function Mods({ store, nav }: { store: Store; nav: Nav }) {
  const [mods, setMods] = useState<InstalledMod[]>([]);
  const refresh = useCallback(() => {
    api.modsInstalled().then(setMods).catch(() => {});
  }, []);
  useEffect(refresh, [refresh, store.launch?.status]);

  const games = store.lib.games.filter((g) => g.installed && !store.settings.hidden.includes(g.key));
  const byGame = (key: string) => mods.filter((m) => m.gameKey === key);

  return (
    <section>
      <h1 className="page-title">المودات</h1>
      <p className="lead">
        افتح أي لعبة ← «تصفح المودات» ← ثبّت بضغطة. تقدر تعطّل أي مود وترجع ملفات اللعبة الأصلية، أو تحذفه نهائياً. الألعاب الأونلاين محمية من المودات.
      </p>
      {!store.settings.nexusKey && (
        <div className="notice">التصفح شغّال بدون أي شي. للتثبيت أضف مفتاح Nexus Mods المجاني من الإعدادات.</div>
      )}
      <div className="grid">
        {games.map((g) => {
          const n = byGame(g.key);
          return (
            <div key={g.key} className="gc" onClick={() => nav.go({ page: "game", key: g.key, title: g.title, tab: n.length ? "installed" : "mods" })}>
              <Art game={g} kind="cover" art={store.lib.art[g.key]} custom={store.settings.customCovers[g.key]} className="poster">
                {n.length > 0 && <span className="st"><Icon name="puzzle" size={12} /> {num(n.filter((m) => m.enabled).length)}/{num(n.length)}</span>}
              </Art>
              <b className="en">{g.title}</b>
              <small>{n.length ? `${num(n.length)} مود` : "تصفح المودات"}</small>
            </div>
          );
        })}
      </div>
    </section>
  );
}
