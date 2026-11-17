import { PLATFORMS, num } from "../lib/platforms";
import type { Store } from "../lib/useLibrary";

export default function Platforms({ store }: { store: Store }) {
  const { lib, settings, saveSettings } = store;
  const steamAccounts = lib.accounts.filter((a) => a.platform === "steam");
  const banned = settings.codBannedAccounts;

  const toggleBanned = (id: string) => {
    const next = banned.includes(id) ? banned.filter((b) => b !== id) : [...banned, id];
    saveSettings({ ...settings, codBannedAccounts: next }, { rescan: true });
  };

  return (
    <section>
      <h1 className="page-title">المنصات والحسابات</h1>
      <p className="lead">ساحة تكتشف المنصات المثبتة على جهازك تلقائياً وتجمع ألعابها في مكتبة وحدة.</p>

      <div className="lg">
        {lib.platforms.map((p) => (
          <div key={p.id} className="lc glass">
            <div className="lc-h">
              <div className="lo" style={{ background: PLATFORMS[p.id].bg, color: PLATFORMS[p.id].fg }}>{PLATFORMS[p.id].short}</div>
              <div>
                <h3>{PLATFORMS[p.id].name}</h3>
                <div className={`status ${p.detected ? "ok" : "off"}`}>
                  <span className="dot" />
                  {p.detected ? "متصلة" : "غير مثبتة"}
                </div>
              </div>
            </div>
            {p.installPath && <div className="path">{p.installPath}</div>}
            <div className="n">
              {num(p.gameCount)} <small>لعبة</small>
            </div>
          </div>
        ))}
      </div>

      <h2 className="section-title">حسابات ستيم</h2>
      <p className="lead">
        كل لعبة تشتغل من الحساب اللي يملكها، وساحة تبدّل الحساب تلقائياً. لازم كل حساب يكون مسجّل دخوله مرة في ستيم بخيار «تذكرني».
      </p>
      <div className="accounts">
        {steamAccounts.map((a) => {
          const owned = lib.games.filter((g) => g.ownerAccountId === a.id).length;
          const isBanned = banned.includes(a.id);
          return (
            <div key={a.id} className={`acc glass ${isBanned ? "banned" : ""}`}>
              <div className="av">{a.displayName.slice(0, 1).toUpperCase()}</div>
              <div className="acc-info">
                <b>
                  {a.displayName} {a.active && <span className="tag ok">مسجّل الآن</span>}
                </b>
                <small className="en">{a.loginName} · {num(owned)} لعبة مثبتة</small>
              </div>
              <label className="ban">
                <span>محظور في Call of Duty</span>
                <button className={`tg ${isBanned ? "on danger" : ""}`} onClick={() => toggleBanned(a.id)} aria-pressed={isBanned} />
              </label>
            </div>
          );
        })}
        {steamAccounts.length === 0 && <p className="lead">ما لقينا حسابات ستيم مسجّلة على الجهاز.</p>}
      </div>
      {steamAccounts.length > 0 && banned.length === 0 && (
        <div className="notice">🛡 قاعدة الأمان: حدّد الحساب المحظور، وCall of Duty ما تشتغل أبداً عليه. لين تحدده، تشغيلها مقفل.</div>
      )}
    </section>
  );
}
