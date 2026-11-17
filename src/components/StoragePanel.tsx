import { useState } from "react";
import { api } from "../lib/api";
import { num } from "../lib/platforms";
import type { StorageReport } from "../lib/types";
import { Badge, Icon } from "./ui";

const size = (b: number) => (b >= 1e12 ? `${(b / 1e12).toFixed(2)} TB` : b >= 1e9 ? `${num(Math.round(b / 1e9))} GB` : `${num(Math.round(b / 1e6))} MB`);

/** Disk usage per drive and per game. Computed on demand (big folders take a few seconds). */
export default function StoragePanel() {
  const [r, setR] = useState<StorageReport | null>(null);
  const [loading, setLoading] = useState(false);
  const load = () => {
    setLoading(true);
    api.storageReport().then(setR).finally(() => setLoading(false));
  };

  if (!r)
    return (
      <div className="custom-colors glass">
        <span className="hint">يحسب حجم كل لعبة والمساحة الفاضية في أقراصك.</span>
        <button className="btn2 glass" onClick={load} disabled={loading}>
          {loading ? <><span className="spinner" /> جاري الحساب…</> : "احسب التخزين"}
        </button>
      </div>
    );

  const max = r.games[0]?.bytes || 1;
  return (
    <div className="storage">
      <div className="drives">
        {r.drives.map((d) => {
          const used = d.total - d.free;
          return (
            <div key={d.root} className="drive glass">
              <div className="drive-h">
                <b className="en">{d.root}</b>
                <span>{size(d.free)} فاضية من {size(d.total)}</span>
              </div>
              <div className="dbar">
                <i className="other" style={{ width: `${(used / d.total) * 100}%` }} />
                <i className="games" style={{ width: `${(d.games / d.total) * 100}%` }} />
              </div>
              <div className="legend">
                <span><i className="games" /> ألعاب {size(d.games)}</span>
                <span><i className="other" /> غيرها {size(Math.max(0, used - d.games))}</span>
              </div>
            </div>
          );
        })}
      </div>
      <div className="card glass">
        <h4>
          الألعاب حسب الحجم
          <span className="spacer" />
          <button className="link" onClick={load} disabled={loading}>{loading ? "…" : "تحديث"}</button>
        </h4>
        {r.games.map((g) => (
          <div key={g.key} className="sz-row">
            <Badge platform={g.platform} />
            <span className="en sz-title" title={g.title}>{g.title}</span>
            <div className="bar"><i style={{ width: `${(g.bytes / max) * 100}%` }} /></div>
            <b className="en">{size(g.bytes)}</b>
            <button className="icon-btn glass sm" title="افتح المجلد" onClick={() => api.openFolder(g.dir)}>
              <Icon name="external" size={16} />
            </button>
          </div>
        ))}
        <div className="row"><span>المودات المثبتة (نسخ ساحة)</span><b className="en">{size(r.modsBytes)}</b></div>
        <div className="row"><span>صور البروفايل والبوسترات</span><b className="en">{size(r.mediaBytes)}</b></div>
      </div>
    </div>
  );
}
