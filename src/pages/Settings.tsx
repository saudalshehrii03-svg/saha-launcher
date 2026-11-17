import { useEffect, useState } from "react";
import { Icon } from "../components/ui";
import { api } from "../lib/api";
import ThemeEditor from "../components/ThemeEditor";
import StoragePanel from "../components/StoragePanel";
import type { Store } from "../lib/useLibrary";
import { checkForUpdate, installUpdate } from "../components/UpdateBanner";

export default function Settings({ store }: { store: Store }) {
  const { lib, settings, saveSettings, fetchArt } = store;
  const [info, setInfo] = useState<string>("");
  const [gridKey, setGridKey] = useState(settings.steamgriddbKey);
  const [steamKey, setSteamKey] = useState(settings.steamApiKey);
  const [nexusKey, setNexusKey] = useState(settings.nexusKey);
  const [upd, setUpd] = useState("");
  useEffect(() => setSteamKey(settings.steamApiKey), [settings.steamApiKey]);
  useEffect(() => setNexusKey(settings.nexusKey), [settings.nexusKey]);

  const saveSteamKey = async () => {
    await saveSettings({ ...settings, steamApiKey: steamKey.trim() });
    store.notify("steam", "جاري جلب كل ألعاب حساباتك…", "running");
    store.syncOwned().then(() => store.notify("steam", "تم: كل ألعابك ظهرت في المكتبة", "done")).catch((e) => store.notify("steam", String(e), "error"));
  };
  const checkUpdate = async () => {
    setUpd("جاري الفحص…");
    const u = await checkForUpdate();
    if (!u) return setUpd("عندك آخر نسخة.");
    setUpd(`نسخة ${u.version} متوفرة، جاري التحديث…`);
    installUpdate(u, (p) => setUpd(`جاري التنزيل… ${p}%`)).catch((e) => setUpd(String(e)));
  };
  useEffect(() => {
    api.appInfo().then((i) => setInfo(`${i.name} ${i.version}`)).catch(() => setInfo("وضع المتصفح (بدون Tauri)"));
  }, []);
  useEffect(() => setGridKey(settings.steamgriddbKey), [settings.steamgriddbKey]);

  const steamAccounts = lib.accounts.filter((a) => a.platform === "steam");
  const steamGames = lib.games.filter((g) => g.platform === "steam");

  const setPinned = (key: string, id: string) => {
    const pinned = { ...settings.pinnedAccounts };
    if (id) pinned[key] = id;
    else delete pinned[key];
    saveSettings({ ...settings, pinnedAccounts: pinned }, { rescan: true });
  };

  const saveGridKey = async () => {
    await saveSettings({ ...settings, steamgriddbKey: gridKey.trim() });
    fetchArt();
  };

  return (
    <section>
      <h1 className="page-title">الإعدادات</h1>

      <h2 className="section-title"><Icon name="palette" /> الألوان</h2>
      <p className="lead">غيّر الألوان والتوهج والزجاج، وشوف المعاينة قبل التطبيق.</p>
      <ThemeEditor store={store} />

      <h2 className="section-title">نمط الشاشة</h2>
      <div className="themes">
        {[["glass", "زجاجي مع توهج"], ["amoled", "AMOLED أسود كامل"]].map(([id, label]) => (
          <button key={id} className={`theme glass ${(settings.mode || "glass") === id ? "on" : ""}`} onClick={() => saveSettings({ ...settings, mode: id })}>
            <span className={`swatch mode-${id}`} />
            {label}
          </button>
        ))}
      </div>

      <h2 className="section-title">كل ألعاب ستيم</h2>
      <p className="lead">
        عشان تظهر كل الألعاب اللي تملكها (مو بس المثبتة) من حساباتك كلها: خذ مفتاح Steam Web API مجاني من
        <button className="link inline" onClick={() => api.openExternal("https://steamcommunity.com/dev/apikey")}> steamcommunity.com/dev/apikey </button>
        وحطه هنا. لازم «تفاصيل الألعاب» تكون عامة في خصوصية كل حساب.
      </p>
      <div className="custom-colors glass">
        <input className="text-input en" type="password" placeholder="Steam Web API key" value={steamKey} onChange={(e) => setSteamKey(e.target.value)} />
        <button className="btn2 glass" onClick={saveSteamKey} disabled={!steamKey.trim()}>حفظ وجلب الألعاب</button>
      </div>

      <h2 className="section-title">المودات (Nexus Mods)</h2>
      <p className="lead">
        التصفح شغال بدون مفتاح. للتثبيت بضغطة: سجّل في Nexus Mods (مجاني) وخذ «Personal API Key» من
        <button className="link inline" onClick={() => api.openExternal("https://www.nexusmods.com/settings/api-keys")}> صفحة المفاتيح </button>.
      </p>
      <div className="custom-colors glass">
        <input className="text-input en" type="password" placeholder="Nexus Mods API key" value={nexusKey} onChange={(e) => setNexusKey(e.target.value)} />
        <button className="btn2 glass" onClick={() => saveSettings({ ...settings, nexusKey: nexusKey.trim() })} disabled={nexusKey.trim() === settings.nexusKey}>حفظ</button>
      </div>

      <h2 className="section-title">أثناء اللعب</h2>
      <div className="pins glass">
        <div className="pin-row">
          <span>
            صغّر ساحة لما تبدأ اللعبة، وارجعها لما تقفلها
            <small className="hint">ساحة تحسب وقت لعبك بنفسها لكل المنصات، حتى اللي ما تعطي وقت اللعب (Epic، Rockstar، Battle.net…).</small>
          </span>
          <button className={`tg ${settings.minimizeOnPlay ? "on" : ""}`} onClick={() => saveSettings({ ...settings, minimizeOnPlay: !settings.minimizeOnPlay })} aria-pressed={settings.minimizeOnPlay} />
        </div>
      </div>

      <h2 className="section-title">الأغلفة</h2>
      <p className="lead">
        أغلفة ألعاب ستيم تجي من ستيم. باقي الألعاب: لو اللعبة موجودة في متجر ستيم تاخذ غلافها تلقائياً، وإذا لا تقدر تضيف مفتاح SteamGridDB المجاني
        (steamgriddb.com ← Preferences ← API) عشان تجيب أغلفة لكل شي.
      </p>
      <div className="custom-colors glass">
        <input className="text-input en" type="password" placeholder="SteamGridDB API key" value={gridKey} onChange={(e) => setGridKey(e.target.value)} />
        <button className="btn2 glass" onClick={saveGridKey} disabled={gridKey.trim() === settings.steamgriddbKey}>حفظ وجلب الأغلفة</button>
      </div>

      <h2 className="section-title">حساب مفروض لكل لعبة</h2>
      <p className="lead">«تلقائي» يعني الحساب اللي يملك اللعبة. الحسابات المحظورة في Call of Duty ما تظهر لألعابها.</p>
      <div className="pins glass">
        {steamGames.map((g) => {
          const options = steamAccounts.filter((a) => !(g.guarded && settings.codBannedAccounts.includes(a.id)));
          return (
            <div key={g.key} className="pin-row">
              <span className="en">{g.guarded ? "🛡 " : ""}{g.title}</span>
              <select value={settings.pinnedAccounts[g.key] ?? ""} onChange={(e) => setPinned(g.key, e.target.value)}>
                <option value="">تلقائي</option>
                {options.map((a) => (
                  <option key={a.id} value={a.id}>{a.displayName} ({a.loginName})</option>
                ))}
              </select>
            </div>
          );
        })}
        {steamGames.length === 0 && <p className="lead">ما فيه ألعاب ستيم.</p>}
      </div>

      <h2 className="section-title">التخزين</h2>
      <StoragePanel />

      <h2 className="section-title">التحديثات</h2>
      <div className="custom-colors glass">
        <span>الإصدار الحالي: <b className="en">{info}</b></span>
        <span className="hint">{upd || "ساحة تتحدّث تلقائياً من GitHub، تنبيه يطلع لك لما ينزل تحديث."}</span>
        <button className="btn2 glass" onClick={checkUpdate}>تحقق الحين</button>
      </div>
    </section>
  );
}
