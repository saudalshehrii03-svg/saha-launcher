import { useEffect, useState } from "react";
import { Icon } from "../components/ui";
import { api } from "../lib/api";
import type { StoreItem } from "../lib/types";
import type { Store } from "../lib/useLibrary";
import type { Nav } from "../App";

const SECTIONS: [string, string][] = [
  ["new_releases", "جديد"],
  ["top_sellers", "الأكثر مبيعاً"],
  ["specials", "العروض"],
  ["coming_soon", "قادمة قريباً"],
];

export default function StorePage({ store, nav }: { store: Store; nav: Nav }) {
  const [data, setData] = useState<Record<string, StoreItem[]> | null>(null);
  const [sec, setSec] = useState("new_releases");
  const [q, setQ] = useState("");
  const [results, setResults] = useState<StoreItem[] | null>(null);

  useEffect(() => {
    api.storeFeatured().then(setData).catch(() => setData({}));
  }, []);
  useEffect(() => {
    if (!q.trim()) return setResults(null);
    const t = setTimeout(() => api.storeSearch(q).then(setResults).catch(() => setResults([])), 350);
    return () => clearTimeout(t);
  }, [q]);

  const owned = new Set(store.lib.games.filter((g) => g.platform === "steam").map((g) => g.appId));
  const items = results ?? data?.[sec] ?? [];
  const featured = !results ? items[0] : undefined;

  return (
    <section>
      <div className="topbar">
        <label className="search glass">
          <Icon name="search" size={19} />
          <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="ابحث في متجر ستيم" />
        </label>
      </div>
      <h1 className="page-title">المتجر</h1>
      {!results && (
        <div className="chips">
          {SECTIONS.map(([id, label]) => (
            <button key={id} className={`chip ${sec === id ? "on" : "glass"}`} onClick={() => setSec(id)}>{label}</button>
          ))}
        </div>
      )}
      {!data && <div className="loading glass"><span className="spinner" /> جاري تحميل المتجر…</div>}
      {featured && (
        <button className="store-hero glass" onClick={() => nav.go({ page: "game", appId: featured.appId, title: featured.name })}>
          <img src={featured.image} alt="" />
          <div>
            <span className="kicker">{SECTIONS.find((s) => s[0] === sec)?.[1]}</span>
            <h2 className="en">{featured.name}</h2>
            <Price item={featured} owned={owned.has(String(featured.appId))} />
          </div>
        </button>
      )}
      <div className="store-grid" key={sec + (results ? "s" : "")}>
        {items.slice(featured ? 1 : 0).map((it, i) => (
          <button key={it.appId} className="store-card glass" style={{ animationDelay: `${Math.min(i, 20) * 25}ms` }} onClick={() => nav.go({ page: "game", appId: it.appId, title: it.name })}>
            <img src={it.image} alt="" loading="lazy" />
            <b className="en">{it.name}</b>
            <Price item={it} owned={owned.has(String(it.appId))} />
          </button>
        ))}
      </div>
      {results && results.length === 0 && <p className="lead">ما فيه نتائج.</p>}
    </section>
  );
}

function Price({ item, owned }: { item: StoreItem; owned: boolean }) {
  if (owned) return <span className="price owned">في مكتبتك</span>;
  return (
    <span className="price">
      {item.discount > 0 && <span className="off-badge">-{item.discount}%</span>}
      {item.price ?? ""}
    </span>
  );
}
