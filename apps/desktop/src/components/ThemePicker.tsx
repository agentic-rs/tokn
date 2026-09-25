import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "../lib/theme";
import type { ThemePreference } from "../lib/types";

const options: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: "system", label: "System", icon: Monitor },
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
];

export function ThemePicker() {
  const { preference, setPreference } = useTheme();
  return (
    <div className="theme-picker" role="group" aria-label="Appearance">
      {options.map(({ value, label, icon: Icon }) => (
        <button
          key={value}
          type="button"
          aria-label={`${label} theme`}
          aria-pressed={preference === value}
          title={`${label} theme`}
          onClick={() => setPreference(value)}
        >
          <Icon size={16} />
          <span>{label}</span>
        </button>
      ))}
    </div>
  );
}
