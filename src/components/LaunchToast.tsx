import type { Store } from "../lib/useLibrary";
import { Icon } from "./ui";

export default function LaunchToast({ store }: { store: Store }) {
  const l = store.launch;
  if (!l) return null;
  const game = store.lib.games.find((g) => g.key === l.key);
  return (
    <div className={`toast glass ${l.status}`}>
      {l.status === "running" ? <span className="spinner" /> : <Icon name={l.status === "done" ? "check" : "x"} />}
      <div>
        <b>{game?.title ?? "تشغيل"}</b>
        <small>{l.message}</small>
      </div>
      {l.status !== "running" && (
        <button className="toast-x" onClick={store.dismissLaunch} aria-label="إغلاق">
          <Icon name="x" size={16} />
        </button>
      )}
    </div>
  );
}
