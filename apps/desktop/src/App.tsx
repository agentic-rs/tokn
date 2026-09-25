import { useState } from "react";
import {
  Activity,
  Boxes,
  GitBranch,
  History as HistoryIcon,
} from "lucide-react";
import { Overview } from "./pages/Overview";
import { Accounts } from "./pages/Accounts";
import { Routing } from "./pages/Routing";
import { Inspect } from "./pages/Inspect";
import { ThemePicker } from "./components/ThemePicker";

const pages = [
  { title: "Overview", icon: Activity, view: Overview },
  { title: "Accounts", icon: Boxes, view: Accounts },
  { title: "Routing", icon: GitBranch, view: Routing },
  { title: "Inspect", icon: HistoryIcon, view: Inspect },
];
export default function App() {
  const [active, setActive] = useState("Overview");
  return (
    <div className="app-shell">
      <aside>
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
          <ThemePicker />
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
