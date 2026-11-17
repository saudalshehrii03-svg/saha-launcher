import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Icon } from "./ui";

/** Checks GitHub Releases on start (and every 6h); one click installs and restarts. */
export async function checkForUpdate(): Promise<Update | null> {
  try {
    return await check();
  } catch {
    return null; // offline / dev build
  }
}

export async function installUpdate(u: Update, onProgress?: (pct: number) => void) {
  let total = 0;
  let got = 0;
  await u.downloadAndInstall((e) => {
    if (e.event === "Started") total = e.data.contentLength ?? 0;
    if (e.event === "Progress") {
      got += e.data.chunkLength;
      if (total) onProgress?.(Math.round((got / total) * 100));
    }
  });
  await relaunch();
}

export default function UpdateBanner() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [pct, setPct] = useState<number | null>(null);
  const [err, setErr] = useState("");

  useEffect(() => {
    const run = () => checkForUpdate().then((u) => u && setUpdate(u));
    run();
    const t = setInterval(run, 6 * 3600 * 1000);
    return () => clearInterval(t);
  }, []);

  if (!update) return null;
  return (
    <div className="update glass">
      <Icon name="sparkle" />
      <div>
        <b>تحديث جديد · {update.version}</b>
        <small>{err || (pct === null ? "ميزات جديدة جاهزة. التحديث يتركّب ويعيد تشغيل ساحة." : `جاري التنزيل… ${pct}%`)}</small>
      </div>
      <button
        className="play small"
        disabled={pct !== null}
        onClick={() => {
          setPct(0);
          installUpdate(update, setPct).catch((e) => {
            setErr(String(e));
            setPct(null);
          });
        }}
      >
        حدّث الآن
      </button>
      <button className="toast-x" onClick={() => setUpdate(null)} aria-label="لاحقاً">
        <Icon name="x" size={16} />
      </button>
    </div>
  );
}
