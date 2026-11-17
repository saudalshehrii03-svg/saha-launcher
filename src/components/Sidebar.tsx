export type PageId = "library" | "platforms" | "mods" | "settings";

const ITEMS: { id: PageId; label: string }[] = [
  { id: "library", label: "المكتبة" },
  { id: "mods", label: "المودات" },
  { id: "platforms", label: "المنصات والحسابات" },
  { id: "settings", label: "الإعدادات" },
];

export default function Sidebar({ page, onNavigate }: { page: PageId; onNavigate: (p: PageId) => void }) {
  return (
    <aside className="sidebar glass">
      <div className="logo">
        <span className="logo-mark" />
        <div>
          <b>ساحة</b>
          <small>SAHA LAUNCHER</small>
        </div>
      </div>
      <nav>
        {ITEMS.map((it) => (
          <button key={it.id} className={`nav-item ${page === it.id ? "on" : ""}`} onClick={() => onNavigate(it.id)}>
            {it.label}
          </button>
        ))}
      </nav>
    </aside>
  );
}
