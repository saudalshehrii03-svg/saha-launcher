import { useState } from "react";
import Sidebar, { type PageId } from "./components/Sidebar";
import Library from "./pages/Library";
import Platforms from "./pages/Platforms";
import Mods from "./pages/Mods";
import Settings from "./pages/Settings";

export default function App() {
  const [page, setPage] = useState<PageId>("library");
  return (
    <div className="shell">
      <Sidebar page={page} onNavigate={setPage} />
      <main className="main">
        {page === "library" && <Library />}
        {page === "platforms" && <Platforms />}
        {page === "mods" && <Mods />}
        {page === "settings" && <Settings />}
      </main>
    </div>
  );
}
