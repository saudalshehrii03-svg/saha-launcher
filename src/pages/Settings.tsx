import { useEffect, useState } from "react";
import { api } from "../lib/api";

export default function Settings() {
  const [info, setInfo] = useState<string>("");
  useEffect(() => {
    api.appInfo().then((i) => setInfo(`${i.name} ${i.version}`)).catch(() => setInfo("وضع المتصفح (بدون Tauri)"));
  }, []);
  return (
    <section>
      <h1 className="page-title">الإعدادات</h1>
      <div className="empty glass">
        <p>الإصدار: {info}</p>
      </div>
    </section>
  );
}
