import { mediaUrl } from "../lib/api";
import { PLATFORMS, num } from "../lib/platforms";
import type { Filter, Store } from "../lib/useLibrary";
import { Icon } from "./ui";

export type PageId = "library" | "store" | "profile" | "platforms" | "mods" | "settings" | "game";

type Props = { page: PageId; onNavigate: (p: PageId) => void; store: Store };

export default function Sidebar({ page, onNavigate, store }: Props) {
  const { lib, settings, filter, setFilter } = store;
  const visible = lib.games.filter((g) => !settings.hidden.includes(g.key));
  const favs = visible.filter((g) => settings.favorites.includes(g.key)).length;
  const p = settings.profile;

  // Library entries just switch the filter; the page stays the same.
  const show = (f: Filter) => {
    setFilter(f);
    if (page !== "library") onNavigate("library");
  };
  const on = (f: Filter) => (page === "library" && filter === f ? "on" : "");
  const item = (id: PageId, icon: string, label: string) => (
    <button className={`nav-item ${page === id ? "on" : ""}`} onClick={() => onNavigate(id)}>
      <Icon name={icon} size={19} />
      {label}
    </button>
  );

  return (
    <aside className="sidebar glass">
      <button className="logo" onClick={() => onNavigate("profile")} title="البروفايل">
        {p.avatar ? <img className="logo-avatar" src={mediaUrl(p.avatar)} alt="" /> : <span className="logo-mark"><Icon name="play" size={22} /></span>}
        <div>
          <b>{p.name || "ساحة"}</b>
          <small>SAHA LAUNCHER</small>
        </div>
      </button>
      <nav>
        <button className={`nav-item ${on("all")}`} onClick={() => show("all")}>
          <Icon name="grid" size={19} />
          كل الألعاب
          {visible.length > 0 && <span className="count">{num(visible.length)}</span>}
        </button>
        <button className={`nav-item ${on("favorites")}`} onClick={() => show("favorites")}>
          <Icon name="heart" size={19} />
          المفضلة
          {favs > 0 && <span className="count">{num(favs)}</span>}
        </button>
        {item("store", "store", "المتجر")}
        {item("mods", "puzzle", "المودات")}
        {item("profile", "user", "البروفايل")}
      </nav>
      <div className="nav-label">المنصات المتصلة</div>
      <nav>
        {lib.platforms
          .filter((pl) => pl.detected)
          .map((pl) => (
            <button key={pl.id} className={`nav-item ${on(pl.id)}`} onClick={() => show(pl.id)}>
              <span className="pd" style={{ background: PLATFORMS[pl.id].bg, color: PLATFORMS[pl.id].fg }}>
                {PLATFORMS[pl.id].short}
              </span>
              {PLATFORMS[pl.id].name}
              <span className="count">{num(visible.filter((g) => g.platform === pl.id).length)}</span>
            </button>
          ))}
        <button className={`nav-item ${page === "platforms" ? "on" : ""}`} onClick={() => onNavigate("platforms")}>
          <span className="pd plus">+</span>
          المنصات والحسابات
        </button>
      </nav>
      <button className={`nav-item bottom ${page === "settings" ? "on" : ""}`} onClick={() => onNavigate("settings")}>
        <Icon name="gear" size={19} />
        الإعدادات
      </button>
    </aside>
  );
}
