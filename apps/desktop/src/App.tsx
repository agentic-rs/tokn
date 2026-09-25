import { useState } from "react";
import {
  Activity,
  Boxes,
  GitBranch,
  History as HistoryIcon,
} from "lucide-react";
import { Overview } from "./pages/Overview";
import { Providers } from "./pages/Providers";
import { Routing } from "./pages/Routing";
import { Inspect } from "./pages/Inspect";
const pages = [
  { title: "Overview", icon: Activity, view: Overview },
  { title: "Providers", icon: Boxes, view: Providers },
  { title: "Routing", icon: GitBranch, view: Routing },
  { title: "Inspect", icon: HistoryIcon, view: Inspect },
];
export default function App() {
  const [active, setActive] = useState("Overview");
  return (
    <div className="app-shell">
      <aside>
        <div className="brand">
          <span className="brand-symbol">t</span>tokn
          <span className="desktop-label">DESKTOP</span>
        </div>
        <div className="nav-label">WORKSPACE</div>
        <nav aria-label="Main navigation">
          {pages.map((page) => (
            <button
              className={active === page.title ? "active" : ""}
              key={page.title}
              onClick={() => setActive(page.title)}
              aria-current={active === page.title ? "page" : undefined}
            >
              <page.icon size={18} />
              {page.title}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <span className="local-dot" />
          Local workspace<small>Tokn · 0.2.4</small>
        </div>
      </aside>
      <main>
        {pages.map((page) => (
          <div key={page.title} hidden={active !== page.title}>
            <page.view />
          </div>
        ))}
      </main>
    </div>
  );
}
