import { useCallback, useEffect, useState } from "react";
import { Art, Badge, Icon, ImagePick } from "../components/ui";
import { api, mediaUrl, nexusGames } from "../lib/api";
import { ago, gb, hours, num } from "../lib/platforms";
import type { Details, Game, InstalledMod, ModInfo, ModSupport } from "../lib/types";
import type { Store } from "../lib/useLibrary";
import type { Nav, View } from "../App";

type GameView = Extract<View, { page: "game" }>;
type Tab = "info" | "mods" | "installed";

export default function GamePage({ store, nav, view, game }: { store: Store; nav: Nav; view: GameView; game?: Game }) {
  const title = game?.title ?? view.title;
  const steamId = game?.platform === "steam" ? Number(game.appId) : view.appId;
  const [tab, setTab] = useState<Tab>(view.tab ?? "info");
  const [details, setDetails] = useState<Details | null>(null);
  const [support, setSupport] = useState<ModSupport | null>(null);
  const [installed, setInstalled] = useState<InstalledMod[]>([]);
  const [openMod, setOpenMod] = useState<ModInfo | null>(null);

  useEffect(() => {
    api.gameDetails(title, steamId).then(setDetails).catch(() => setDetails(null));
    api.modSupport(title).then((s) => {
      setSupport(s);
      if (s.domain && game) nexusGames.set(s.domain, game.key); // so nxm:// links know the game
    });
  }, [title, steamId, game]);

  const refreshInstalled = useCallback(() => {
    if (game) api.modsInstalled().then((all) => setInstalled(all.filter((m) => m.gameKey === game.key)));
  }, [game]);
  useEffect(refreshInstalled, [refreshInstalled, store.launch?.status]);

  const bg = details?.background ?? details?.screenshots[0];
  const busy = store.launch?.status === "running";

  return (
    <section className="gp">
      <div className="gp-bg" style={bg ? { backgroundImage: `url(${bg})` } : undefined} />
      <div className="gp-top">
        <button className="icon-btn glass" onClick={openMod ? () => setOpenMod(null) : nav.back} title="رجوع">
          <Icon name="back" />
        </button>
        <span className="spacer" />
        {game && (
          <button className={`icon-btn glass ${store.settings.favorites.includes(game.key) ? "fav-on" : ""}`} onClick={() => store.toggleIn("favorites", game.key)} title="المفضلة">
            <Icon name={store.settings.favorites.includes(game.key) ? "heart-on" : "heart"} />
          </button>
        )}
      </div>

      <div className="gp-head">
        {game && <Art game={game} kind="cover" art={store.lib.art[game.key]} custom={store.settings.customCovers[game.key]} className="poster gp-poster" />}
        <div className="gp-title">
          <div className="kicker">
            {game ? <Badge platform={game.platform} full /> : <span className="badge glass">متجر Steam</span>}
            {details?.goty && <span className="badge goty"><Icon name="trophy" size={14} /> لعبة السنة</span>}
            {game && !game.installed && <span className="badge glass">غير مثبتة</span>}
          </div>
          <h1 className="en">{details?.name ?? title}</h1>
          <div className="stats">
            {details?.currentPlayers != null && <div><b><span className="live-dot" />{num(details.currentPlayers)}</b>يلعبون الحين</div>}
            {game?.playtimeMinutes ? <div><b>{hours(game.playtimeMinutes)}</b>وقت لعبك</div> : null}
            {game?.lastPlayed ? <div><b>{ago(game.lastPlayed)}</b>آخر تشغيل</div> : null}
            {details?.metacritic ? <div><b>{num(details.metacritic)}</b>Metacritic</div> : null}
            {game?.sizeBytes ? <div><b>{gb(game.sizeBytes)}</b>الحجم</div> : null}
          </div>
          <div className="gp-actions">
            {game ? (
              <button className="play" disabled={busy} onClick={() => store.play(game.key)}>
                <Icon name={game.installed ? "play" : "download"} size={22} /> {game.installed ? "العب الآن" : "ثبّت عبر ستيم"}
              </button>
            ) : (
              steamId && (
                <button className="play" onClick={() => api.openExternal(`steam://store/${steamId}`)}>
                  <Icon name="store" size={20} /> افتح في ستيم
                </button>
              )
            )}
            {details?.deals[0] && (
              <button className="btn2 glass big" onClick={() => api.openExternal(details.deals[0].url)}>
                <Icon name="tag" size={18} /> أرخص سعر: {details.deals[0].price} · {details.deals[0].store}
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="tabs glass">
        {([["info", "معلومات"], ["mods", "تصفح المودات"], ["installed", `المودات المثبتة${installed.length ? ` · ${num(installed.length)}` : ""}`]] as [Tab, string][]).map(([id, label]) => (
          <button key={id} className={tab === id ? "on" : ""} onClick={() => { setTab(id); setOpenMod(null); }}>{label}</button>
        ))}
      </div>

      <div className="tab-body" key={tab + (openMod?.modId ?? "")}>
        {tab === "info" && <Info details={details} game={game} store={store} />}
        {tab === "mods" && !openMod && <ModBrowser support={support} onOpen={setOpenMod} />}
        {tab === "mods" && openMod && <ModView gameTitle={title} mod={openMod} game={game} store={store} support={support} installed={installed} onDone={refreshInstalled} />}
        {tab === "installed" && <Installed mods={installed} store={store} onChange={refreshInstalled} game={game} />}
      </div>
    </section>
  );
}

function Info({ details, game, store }: { details: Details | null; game?: Game; store: Store }) {
  if (!details) return <div className="loading glass"><span className="spinner" /> جاري جلب معلومات اللعبة…</div>;
  const custom = game ? store.settings.customCovers[game.key] ?? [] : [];
  const setCovers = (paths: string[]) => {
    if (!game) return;
    const all = { ...store.settings.customCovers };
    if (paths.length) all[game.key] = paths;
    else delete all[game.key];
    store.saveSettings({ ...store.settings, customCovers: all });
  };
  return (
    <div className="info-grid">
      <div className="col">
        {details.description && <p className="desc glass">{details.description}</p>}
        {details.screenshots.length > 0 && (
          <div className="shots">
            {details.screenshots.map((s) => <img key={s} src={s} alt="" loading="lazy" />)}
          </div>
        )}
      </div>
      <div className="col side">
        {details.deals.length > 0 && (
          <div className="card glass">
            <h4><Icon name="tag" size={18} /> أرخص المتاجر الموثوقة</h4>
            {details.deals.map((d) => (
              <button key={d.url} className="deal" onClick={() => api.openExternal(d.url)}>
                <span>{d.store}</span>
                {d.savings > 0 && <span className="off-badge">-{d.savings}%</span>}
                <b className="en">{d.price}</b>
                <Icon name="external" size={15} />
              </button>
            ))}
            <small className="hint">الأسعار بالدولار من متاجر رسمية معتمدة (عبر CheapShark).{details.price ? ` سعر ستيم السعودي: ${details.price}` : ""}</small>
          </div>
        )}
        {details.awards.length > 0 && (
          <div className="card glass">
            <h4><Icon name="trophy" size={18} /> الجوائز {details.goty && <span className="badge goty">GOTY</span>}</h4>
            <ul className="awards">{details.awards.slice(0, 12).map((a) => <li key={a}>{a}</li>)}</ul>
          </div>
        )}
        <div className="card glass">
          <h4>تفاصيل</h4>
          {details.genres.length > 0 && <div className="row"><span>النوع</span>{details.genres.join("، ")}</div>}
          {details.developers.length > 0 && <div className="row"><span>المطوّر</span><span className="en">{details.developers.join(", ")}</span></div>}
          {details.publishers.length > 0 && <div className="row"><span>الناشر</span><span className="en">{details.publishers.join(", ")}</span></div>}
          {details.releaseDate && <div className="row"><span>الإصدار</span>{details.releaseDate}</div>}
          {details.price && <div className="row"><span>السعر في ستيم</span>{details.price}</div>}
        </div>
        {game && (
          <div className="card glass">
            <h4><Icon name="image" size={18} /> بوسترات مخصصة</h4>
            <p className="hint">حط بوستر من عندك (GIF وWebP المتحركة تشتغل). أكثر من صورة = تتبدّل كل ثانية.</p>
            {custom.length > 0 && (
              <div className="thumbs">
                {custom.map((p) => (
                  <button key={p} className="thumb" onClick={() => setCovers(custom.filter((x) => x !== p))} title="إزالة">
                    <img src={mediaUrl(p)} alt="" />
                    <span><Icon name="x" size={14} /></span>
                  </button>
                ))}
              </div>
            )}
            <div className="row-btns">
              <ImagePick label={<><Icon name="image" size={16} /> أضف بوسترات</>} multiple onPick={(p) => setCovers([...custom, ...p])} />
              {custom.length > 0 && <button className="btn2 glass" onClick={() => setCovers([])}>رجّع الرسمي</button>}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ModBrowser({ support, onOpen }: { support: ModSupport | null; onOpen: (m: ModInfo) => void }) {
  const [term, setTerm] = useState("");
  const [sort, setSort] = useState("endorsements");
  const [mods, setMods] = useState<ModInfo[] | null>(null);
  const [err, setErr] = useState("");
  const domain = support?.domain;

  useEffect(() => {
    if (!domain) return;
    setMods(null);
    const t = setTimeout(() => {
      api.modsBrowse(domain, term, sort).then(setMods).catch((e) => setErr(String(e)));
    }, 300);
    return () => clearTimeout(t);
  }, [domain, term, sort]);

  if (!support) return <div className="loading glass"><span className="spinner" /> نشيك دعم المودات…</div>;
  if (support.blocked) return <div className="notice danger">🛡 {support.blocked}</div>;
  if (!domain) return <div className="notice">ما لقيت هذي اللعبة في Nexus Mods.</div>;

  return (
    <>
      {support.warning && <div className="notice">⚠ {support.warning}</div>}
      <div className="mod-bar">
        <label className="search glass">
          <Icon name="search" size={18} />
          <input value={term} onChange={(e) => setTerm(e.target.value)} placeholder="ابحث في المودات" />
        </label>
        <div className="seg glass">
          {[["endorsements", "الأعلى تقييماً"], ["downloads", "الأكثر تحميلاً"], ["updatedAt", "المحدّثة"], ["createdAt", "الجديدة"]].map(([id, l]) => (
            <button key={id} className={sort === id ? "on" : ""} onClick={() => setSort(id)}>{l}</button>
          ))}
        </div>
      </div>
      {err && <div className="notice danger">{err}</div>}
      {!mods && !err && <div className="loading glass"><span className="spinner" /> جاري التحميل…</div>}
      <div className="mod-grid">
        {mods?.map((m, i) => (
          <button key={m.modId} className="mod-card glass" onClick={() => onOpen(m)} style={{ animationDelay: `${Math.min(i, 20) * 20}ms` }}>
            <div className="mod-img" style={m.thumbnail ? { backgroundImage: `url(${m.thumbnail})` } : undefined} />
            <div className="mod-tx">
              <b>{m.name}</b>
              <small>{m.summary}</small>
              <div className="mod-meta">
                <span>👍 {num(m.endorsements)}</span>
                <span>⬇ {num(m.downloads)}</span>
                <span className="en">{m.author}</span>
              </div>
            </div>
          </button>
        ))}
      </div>
    </>
  );
}

function ModView({ gameTitle, mod, game, store, support, installed, onDone }: { gameTitle: string; mod: ModInfo; game?: Game; store: Store; support: ModSupport | null; installed: InstalledMod[]; onDone: () => void }) {
  const [full, setFull] = useState<ModInfo>(mod);
  const [explain, setExplain] = useState<Awaited<ReturnType<typeof api.modExplain>> | null>(null);
  useEffect(() => {
    api
      .modPage(mod.domain, mod.modId)
      .then((m) => {
        setFull(m);
        return api.modExplain(m.name, gameTitle, m.summary, m.description);
      })
      .then(setExplain)
      .catch(() => {});
  }, [mod, gameTitle]);
  const have = installed.find((m) => m.modId === mod.modId);
  const canInstall = game?.installed && !support?.blocked;
  const images = [full.picture, ...full.images].filter(Boolean) as string[];

  const install = () => {
    if (!game) return;
    if (!store.settings.nexusKey) {
      store.notify(game.key, "أضف مفتاح Nexus Mods من الإعدادات أول (مجاني).", "error");
      return;
    }
    // Premium: direct. Free: Nexus needs the site button -> nxm:// comes back to Saha.
    store.notify(game.key, "جاري التثبيت…", "running");
    api
      .modInstall(game.key, { domain: mod.domain, modId: mod.modId })
      .then((m) => {
        store.notify(game.key, `تم تثبيت «${m.name}»`, "done");
        onDone();
      })
      .catch((e) => {
        if (!String(e).includes("Mod Manager Download")) return store.notify(game.key, String(e), "error");
        store.notify(game.key, "حسابك المجاني في Nexus يحتاج ضغطة «Mod Manager Download». فتحت لك صفحة الملفات، اضغطها وساحة تكمل التثبيت تلقائياً.", "running");
        api.openExternal(`${mod.url}?tab=files`);
      });
  };

  return (
    <div className="mod-view">
      <div className="mv-gallery">
        {images.length > 0 ? images.map((src) => <img key={src} src={src} alt="" loading="lazy" />) : <div className="mod-img big" />}
      </div>
      <div className="mv-side">
        <div className="card glass">
          <h2>{full.name}</h2>
          <p className="hint">{full.summary}</p>
          <div className="mod-meta big">
            <span>👍 {num(full.endorsements)} تأييد</span>
            <span>⬇ {num(full.downloads)} تحميل</span>
          </div>
          <div className="row"><span>المطوّر</span><span className="en">{full.author}</span></div>
          <div className="row"><span>الإصدار</span><span className="en">{full.version}</span></div>
          <div className="row"><span>آخر تحديث</span>{full.updatedAt.slice(0, 10)}</div>
          <div className="row-btns col">
            {have ? (
              <span className="tag ok">مثبت {have.enabled ? "ومفعّل" : "(معطّل)"}</span>
            ) : canInstall ? (
              <button className="play" onClick={install}><Icon name="download" size={20} /> ثبّت بضغطة</button>
            ) : (
              <span className="hint">{support?.blocked ?? (game ? "ثبّت اللعبة أول عشان تركّب مودات." : "تقدر تتصفح المودات، والتثبيت للألعاب اللي عندك.")}</span>
            )}
            <button className="btn2 glass" onClick={() => api.openExternal(`${full.url}?tab=posts`)}><Icon name="people" size={16} /> آراء الناس والتعليقات</button>
            <button className="btn2 glass" onClick={() => api.openExternal(full.url)}><Icon name="external" size={16} /> صفحة المود في Nexus</button>
          </div>
        </div>
        <div className="card glass explain">
          <h4><Icon name="sparkle" size={18} /> وش يسوي هذا المود؟</h4>
          {!explain ? (
            <p className="hint"><span className="spinner" /> جاري التلخيص بالعربي…</p>
          ) : (
            <>
              <p className="ar-explain">{explain.arabic || full.summary}</p>
              {explain.video ? (
                <button className="video" onClick={() => api.openExternal(explain.video!.url)}>
                  <img src={`https://i.ytimg.com/vi/${explain.video.id}/mqdefault.jpg`} alt="" />
                  <span className="vplay"><Icon name="play" size={20} /></span>
                  <span className="vtitle en">{explain.video.title}</span>
                </button>
              ) : (
                <button className="btn2 glass" onClick={() => api.openExternal(explain.searchUrl)}><Icon name="play" size={16} /> مقاطع شرح على يوتيوب</button>
              )}
              <small className="hint">الشرح مترجم آلياً من وصف المود.</small>
            </>
          )}
        </div>
        {full.description && <pre className="desc glass">{full.description}</pre>}
      </div>
    </div>
  );
}

function Installed({ mods, store, onChange, game }: { mods: InstalledMod[]; store: Store; onChange: () => void; game?: Game }) {
  const act = (p: Promise<unknown>, done: string) =>
    p.then(() => { store.notify(game?.key ?? "mods", done, "done"); onChange(); }).catch((e) => store.notify(game?.key ?? "mods", String(e), "error"));
  if (!game) return <div className="notice">المودات المثبتة تظهر للألعاب اللي في مكتبتك.</div>;
  if (mods.length === 0) return <div className="notice">ما فيه مودات مثبتة. تصفّح المودات وثبّت بضغطة.</div>;
  return (
    <div className="installed-list">
      {mods.map((m) => (
        <div key={m.id} className={`im glass ${m.enabled ? "" : "off"}`}>
          <div className="im-img" style={m.picture ? { backgroundImage: `url(${m.picture})` } : undefined} />
          <div className="im-tx">
            <b>{m.name}</b>
            <small className="en">v{m.version} · {num(m.files.length)} ملف</small>
          </div>
          <label className="ban">
            <span>{m.enabled ? "مفعّل" : "معطّل"}</span>
            <button className={`tg ${m.enabled ? "on" : ""}`} onClick={() => act(api.modSetEnabled(m.id, !m.enabled), m.enabled ? "تم التعطيل، ورجعت ملفات اللعبة الأصلية" : "تم التفعيل")} />
          </label>
          <button className="icon-btn glass sm" title="حذف نظيف" onClick={() => act(api.modRemove(m.id), "تم حذف المود نهائياً")}>
            <Icon name="trash" size={18} />
          </button>
        </div>
      ))}
    </div>
  );
}
