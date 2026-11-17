import { useEffect, useRef, useState } from "react";
import { applyTheme, THEMES } from "../lib/platforms";
import { DEFAULT_THEME, type Store } from "../lib/useLibrary";
import type { Theme } from "../lib/types";
import { Icon } from "./ui";

/**
 * Color editor: changes apply to a live mini preview (sidebar + home screen) first;
 * "Apply" saves them to the whole app.
 */
export default function ThemeEditor({ store }: { store: Store }) {
  const saved = store.settings.theme;
  const [draft, setDraft] = useState<Theme>(saved);
  useEffect(() => setDraft(saved), [saved]);
  const preview = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (preview.current) applyTheme(draft, preview.current);
  }, [draft]);
  const dirty = JSON.stringify(draft) !== JSON.stringify(saved);
  const set = (patch: Partial<Theme>) => setDraft((d) => ({ ...d, ...patch }));
  const games = store.lib.games.filter((g) => g.installed).slice(0, 5);

  return (
    <div className="theme-editor">
      <div className="te-controls">
        <div className="themes">
          {THEMES.map((t) => {
            const on = t.accent === draft.accent && t.accent2 === draft.accent2;
            return (
              <button key={t.name} className={`theme glass ${on ? "on" : ""}`} onClick={() => set({ accent: t.accent, accent2: t.accent2, bg: t.bg ?? DEFAULT_THEME.bg })}>
                <span className="swatch" style={{ background: `linear-gradient(135deg, ${t.accent2}, ${t.accent})` }} />
                {t.name}
              </button>
            );
          })}
        </div>
        <div className="te-grid glass">
          <label>اللون الأساسي<input type="color" value={draft.accent} onChange={(e) => set({ accent: e.target.value })} /></label>
          <label>اللون الثانوي<input type="color" value={draft.accent2} onChange={(e) => set({ accent2: e.target.value })} /></label>
          <label>الخلفية<input type="color" value={draft.bg} onChange={(e) => set({ bg: e.target.value })} /></label>
          <label className="range">
            <span>قوة التوهج <b>{draft.glow}%</b></span>
            <input type="range" min={0} max={200} value={draft.glow} onChange={(e) => set({ glow: +e.target.value })} />
          </label>
          <label className="range">
            <span>شفافية الزجاج <b>{draft.glass}%</b></span>
            <input type="range" min={0} max={200} value={draft.glass} onChange={(e) => set({ glass: +e.target.value })} />
          </label>
          <label className="range">
            <span>ضبابية الزجاج <b>{draft.blur}px</b></span>
            <input type="range" min={0} max={60} value={draft.blur} onChange={(e) => set({ blur: +e.target.value })} />
          </label>
        </div>
        <div className="row-btns">
          <button className="play small" disabled={!dirty} onClick={() => store.saveSettings({ ...store.settings, theme: draft })}>
            <Icon name="check" size={18} /> تطبيق
          </button>
          <button className="btn2 glass" disabled={!dirty} onClick={() => setDraft(saved)}>إلغاء</button>
          <button className="btn2 glass" onClick={() => setDraft(DEFAULT_THEME)}>الافتراضي</button>
        </div>
      </div>

      {/* Live preview: same classes as the app, scoped CSS variables. */}
      <div className="te-preview" ref={preview} aria-label="معاينة">
        <div className="pv-bg" />
        <div className="pv-side glass">
          <div className="pv-logo"><span /><b>ساحة</b></div>
          {["كل الألعاب", "المفضلة", "المتجر", "المودات"].map((t, i) => (
            <div key={t} className={`pv-item ${i === 0 ? "on" : ""}`}>{t}</div>
          ))}
          <small>المنصات المتصلة</small>
          {["Steam", "Epic", "Xbox"].map((t) => <div key={t} className="pv-item">{t}</div>)}
        </div>
        <div className="pv-main">
          <div className="pv-search glass">ابحث في كل ألعابك…</div>
          <div className="pv-hero glass">
            <b>{games[0]?.title ?? "ELDEN RING"}</b>
            <span className="pv-play">▶ العب الآن</span>
          </div>
          <div className="pv-chips">
            <span className="pv-chip on">الكل</span>
            <span className="pv-chip glass">Steam</span>
            <span className="pv-chip glass">Epic</span>
          </div>
          <div className="pv-grid">
            {[0, 1, 2, 3, 4].map((i) => <div key={i} className={`pv-poster ${i === 1 ? "hover" : ""}`} />)}
          </div>
        </div>
      </div>
    </div>
  );
}
