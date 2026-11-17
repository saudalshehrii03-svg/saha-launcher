import { useState } from "react";
import { Art, Icon, ImagePick } from "../components/ui";
import { mediaUrl } from "../lib/api";
import { hours, num } from "../lib/platforms";
import type { Store } from "../lib/useLibrary";
import type { Nav } from "../App";

export default function Profile({ store, nav }: { store: Store; nav: Nav }) {
  const { lib, settings, saveSettings } = store;
  const p = settings.profile;
  const [editing, setEditing] = useState(!p.name);
  const [name, setName] = useState(p.name);
  const [bio, setBio] = useState(p.bio);
  const setProfile = (patch: Partial<typeof p>) => saveSettings({ ...settings, profile: { ...p, ...patch } });

  const games = lib.games.filter((g) => !settings.hidden.includes(g.key));
  const favs = games.filter((g) => settings.favorites.includes(g.key));
  const totalMin = games.reduce((s, g) => s + (g.playtimeMinutes ?? 0), 0);
  const top = [...games].sort((a, b) => (b.playtimeMinutes ?? 0) - (a.playtimeMinutes ?? 0)).slice(0, 5);
  const recent = games.filter((g) => g.lastPlayed).slice(0, 6);
  const platforms = new Set(games.map((g) => g.platform)).size;

  return (
    <section className="profile">
      <div className="pf-header glass" style={p.header ? { backgroundImage: `url(${mediaUrl(p.header)})` } : undefined}>
        <ImagePick className="pf-edit glass" label={<><Icon name="image" size={16} /> غيّر الهيدر</>} onPick={([h]) => setProfile({ header: h })} />
      </div>
      <div className="pf-id">
        <div className="pf-avatar glass">
          {p.avatar ? <img src={mediaUrl(p.avatar)} alt="" /> : <span>{(p.name || "س").slice(0, 1)}</span>}
          <ImagePick className="pf-av-edit" label={<Icon name="image" size={16} />} onPick={([a]) => setProfile({ avatar: a })} />
        </div>
        {editing ? (
          <div className="pf-form glass">
            <input className="text-input" value={name} onChange={(e) => setName(e.target.value)} placeholder="اسمك" />
            <input className="text-input" value={bio} onChange={(e) => setBio(e.target.value)} placeholder="نبذة قصيرة" />
            <button className="play small" onClick={() => { setProfile({ name, bio }); setEditing(false); }}>حفظ</button>
          </div>
        ) : (
          <div className="pf-name">
            <h1>{p.name}</h1>
            <p>{p.bio}</p>
            <button className="link" onClick={() => setEditing(true)}>تعديل</button>
          </div>
        )}
      </div>

      <div className="pf-stats">
        <div className="stat glass"><b>{num(games.length)}</b>لعبة</div>
        <div className="stat glass"><b>{num(games.filter((g) => g.installed).length)}</b>مثبتة</div>
        <div className="stat glass"><b>{hours(totalMin) || "٠"}</b>وقت اللعب</div>
        <div className="stat glass"><b>{num(platforms)}</b>منصات</div>
        <div className="stat glass"><b>{num(favs.length)}</b>مفضلة</div>
      </div>

      <h2 className="section-title"><Icon name="heart" /> ألعابي المفضلة</h2>
      {favs.length === 0 && <p className="lead">اضغط ♥ على أي لعبة عشان تظهر هنا. تقدر تحط لها بوستر متحرك من صفحتها.</p>}
      <div className="grid showcase">
        {favs.map((g) => (
          <div key={g.key} className="gc" onClick={() => nav.go({ page: "game", key: g.key, title: g.title })}>
            <Art game={g} kind="cover" art={lib.art[g.key]} custom={settings.customCovers[g.key]} className="poster" />
            <b className="en">{g.title}</b>
            <small>{hours(g.playtimeMinutes) || "—"}</small>
          </div>
        ))}
      </div>

      <div className="pf-cols">
        <div className="card glass">
          <h4>الأكثر لعباً</h4>
          {top.map((g) => {
            const pct = top[0].playtimeMinutes ? ((g.playtimeMinutes ?? 0) / top[0].playtimeMinutes) * 100 : 0;
            return (
              <div key={g.key} className="bar-row">
                <span className="en">{g.title}</span>
                <div className="bar"><i style={{ width: `${pct}%` }} /></div>
                <small>{hours(g.playtimeMinutes)}</small>
              </div>
            );
          })}
        </div>
        <div className="card glass">
          <h4>لعبت مؤخراً</h4>
          <div className="recent">
            {recent.map((g) => (
              <button key={g.key} onClick={() => nav.go({ page: "game", key: g.key, title: g.title })}>
                <Art game={g} kind="cover" art={lib.art[g.key]} custom={settings.customCovers[g.key]} className="poster mini" />
              </button>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
