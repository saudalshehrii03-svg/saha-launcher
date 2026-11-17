import { useEffect, useMemo, useRef, useState } from "react";
import EmptyState from "../components/EmptyState";
import { Art, Badge, Icon } from "../components/ui";
import { PLATFORMS, ago, gb, hours, num } from "../lib/platforms";
import type { Account, Game } from "../lib/types";
import type { Filter, Store } from "../lib/useLibrary";
import type { Nav } from "../App";

type Show = "installed" | "all";
const readShow = (): Show => {
  try {
    return localStorage.getItem("saha.show") === "all" ? "all" : "installed";
  } catch {
    return "installed";
  }
};

function accountName(accounts: Account[], id?: string) {
  return accounts.find((a) => a.id === id)?.displayName;
}

export default function Library({ store, nav }: { store: Store; nav: Nav }) {
  const { lib, settings, scanning, error, play, launch, rescan, running, filter, setFilter } = store;
  const [q, setQ] = useState("");
  const [pick, setPick] = useState<string>(); // "what should I play?" choice
  const [show, setShowState] = useState<Show>(readShow);
  const setShow = (v: Show) => {
    setShowState(v);
    try {
      localStorage.setItem("saha.show", v);
    } catch {
      /* ignore */
    }
  };
  const search = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        search.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const visible = useMemo(() => lib.games.filter((g) => !settings.hidden.includes(g.key)), [lib.games, settings.hidden]);
  const games = useMemo(() => {
    const base =
      filter === "hidden"
        ? lib.games.filter((g) => settings.hidden.includes(g.key))
        : filter === "favorites"
          ? visible.filter((g) => settings.favorites.includes(g.key))
          : filter === "all"
            ? visible
            : visible.filter((g) => g.platform === filter);
    const term = q.trim().toLowerCase();
    const shown = show === "installed" ? base.filter((g) => g.installed) : base;
    return term ? shown.filter((g) => g.title.toLowerCase().includes(term)) : shown;
  }, [lib.games, visible, settings, filter, q, show]);
  const notInstalled = lib.games.filter((g) => !g.installed).length;

  const hero = visible.find((g) => g.key === pick) ?? visible.find((g) => g.lastPlayed) ?? visible[0];
  const busy = launch?.status === "running";
  const hiddenCount = settings.hidden.length;

  const surprise = () => {
    const pool = visible.filter((g) => g.installed && g.key !== hero?.key);
    if (pool.length) setPick(pool[Math.floor(Math.random() * pool.length)].key);
  };

  const chips: { id: Filter; label: React.ReactNode; count: number }[] = [
    { id: "all", label: "الكل", count: visible.length },
    { id: "favorites", label: <><Icon name="heart" size={15} /> المفضلة</>, count: visible.filter((g) => settings.favorites.includes(g.key)).length },
    ...lib.platforms
      .map((p) => ({ id: p.id as Filter, label: <Badge platform={p.id} />, count: visible.filter((g) => g.platform === p.id).length }))
      .filter((c) => c.count > 0),
  ];
  if (hiddenCount > 0) chips.push({ id: "hidden", label: <><Icon name="eyeOff" size={15} /> المخفية</>, count: hiddenCount });

  return (
    <section>
      <div className="topbar">
        <label className="search glass">
          <Icon name="search" size={19} />
          <input ref={search} value={q} onChange={(e) => setQ(e.target.value)} placeholder="ابحث في كل ألعابك من كل المنصات" />
          <span className="kbd">Ctrl K</span>
        </label>
        <span className="spacer" />
        <button className="icon-btn glass" onClick={surprise} title="وش ألعب؟ اختيار عشوائي">
          <Icon name="dice" />
        </button>
        <button className={`icon-btn glass ${scanning ? "spin" : ""}`} onClick={rescan} title="إعادة الفحص">
          <Icon name="sync" />
        </button>
      </div>

      {error && <EmptyState title="تعذّر فحص المكتبة" text={error} />}
      {!scanning && !error && lib.games.length === 0 && (
        <EmptyState title="ما لقينا ألعاب" text="ثبّت ألعاب من أي منصة، ثم اضغط إعادة الفحص." />
      )}

      {hero && (
        <Art key={hero.key} game={hero} kind="hero" art={lib.art[hero.key]} className="hero" onClick={() => nav.go({ page: "game", key: hero.key, title: hero.title })}>
          <div className="hero-tx">
            <div className="kicker">
              <Badge platform={hero.platform} full />
              {hero.key === pick ? "🎲 اقتراح ساحة لك اليوم" : `تابع من حيث توقفت · آخر تشغيل ${ago(hero.lastPlayed)}`}
            </div>
            <h1 className="en">{hero.title}</h1>
            <div className="stats">
              {hero.playtimeMinutes ? <div><b>{hours(hero.playtimeMinutes)}</b>وقت اللعب</div> : null}
              {hero.sizeBytes ? <div><b>{gb(hero.sizeBytes)}</b>الحجم</div> : null}
              {hero.launchAccountId && <div><b>{accountName(lib.accounts, hero.launchAccountId)}</b>الحساب</div>}
            </div>
            <button className="play" disabled={busy || running.includes(hero.key)} onClick={(e) => { e.stopPropagation(); play(hero.key); }}>
              <Icon name="play" size={22} /> {running.includes(hero.key) ? "شغّالة الحين" : "العب الآن"}
            </button>
          </div>
        </Art>
      )}

      {lib.games.length > 0 && (
        <div className="chips">
          {chips.map((c) => (
            <button key={c.id} className={`chip ${filter === c.id ? "on" : "glass"}`} onClick={() => setFilter(c.id)}>
              {c.label} <span className="chip-n">{num(c.count)}</span>
            </button>
          ))}
          <span className="spacer" />
          <div className="seg glass" role="tablist">
            <button className={show === "installed" ? "on" : ""} onClick={() => setShow("installed")}>المثبتة</button>
            <button className={show === "all" ? "on" : ""} onClick={() => setShow("all")}>
              الكل{notInstalled > 0 ? ` · ${num(notInstalled)} غير مثبتة` : ""}
            </button>
          </div>
        </div>
      )}
      {show === "all" && notInstalled === 0 && !settings.steamApiKey && (
        <div className="notice">عشان تظهر كل ألعاب حساباتك في ستيم (مو بس المثبتة)، أضف مفتاح Steam Web API من الإعدادات.</div>
      )}

      {games.length === 0 && lib.games.length > 0 && (
        <p className="lead">{filter === "favorites" ? "ما فيه مفضلة بعد. اضغط ♥ على أي لعبة." : "ما فيه ألعاب هنا."}</p>
      )}

      <div className="grid" key={filter}>
        {games.map((g, i) => (
          <GameCard key={g.key} index={i} game={g} store={store} busy={busy} onOpen={() => nav.go({ page: "game", key: g.key, title: g.title })} />
        ))}
      </div>
    </section>
  );
}

function GameCard({ game: g, store, busy, index, onOpen }: { game: Game; store: Store; busy: boolean; index: number; onOpen: () => void }) {
  const { lib, settings, launch, running, play, toggleIn } = store;
  const launching = launch?.key === g.key && launch.status === "running";
  const isRunning = running.includes(g.key);
  const fav = settings.favorites.includes(g.key);
  const hidden = settings.hidden.includes(g.key);
  const acc = accountName(lib.accounts, g.launchAccountId);
  const sub = isRunning
    ? "● شغّالة الحين"
    : g.guarded && !g.launchAccountId
      ? "🛡 يحتاج تحديد الحساب المحظور"
      : acc
        ? `${PLATFORMS[g.platform].name} · ${acc}`
        : ago(g.lastPlayed);

  return (
    <div className={`gc ${g.installed ? "" : "dim"} ${launching ? "launching" : ""} ${isRunning ? "running" : ""}`} style={{ animationDelay: `${Math.min(index, 20) * 25}ms` }}>
      <Art game={g} kind="cover" art={lib.art[g.key]} custom={settings.customCovers[g.key]} className="poster" onClick={onOpen}>
        <span className="pf"><Badge platform={g.platform} /></span>
        {g.guarded && <span className="guard" title="محمية بقاعدة الحساب المحظور"><Icon name="shield" size={14} /></span>}
        <div className="card-actions" onClick={(e) => e.stopPropagation()}>
          <button className={fav ? "on" : ""} onClick={() => toggleIn("favorites", g.key)} title={fav ? "إزالة من المفضلة" : "إضافة للمفضلة"}>
            <Icon name={fav ? "heart-on" : "heart"} size={15} />
          </button>
          <button onClick={() => toggleIn("hidden", g.key)} title={hidden ? "إظهار" : "إخفاء من المكتبة"}>
            <Icon name={hidden ? "eye" : "eyeOff"} size={15} />
          </button>
        </div>
        {g.playtimeMinutes ? <span className="st">{hours(g.playtimeMinutes)}</span> : !g.installed && <span className="st">غير مثبتة</span>}
        <button className="pb" disabled={busy || isRunning} onClick={(e) => { e.stopPropagation(); play(g.key); }} aria-label={`${g.installed ? "تشغيل" : "تثبيت"} ${g.title}`}>
          {launching ? <span className="spinner dark" /> : <Icon name={g.installed ? "play" : "download"} size={24} />}
        </button>
      </Art>
      <b className="en" title={g.title}>{fav && <span className="fav-dot">♥ </span>}{g.title}</b>
      <small>{sub}</small>
    </div>
  );
}
