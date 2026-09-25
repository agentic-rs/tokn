import { useEffect, useLayoutEffect, useState } from "react";
import type { ThemePreference } from "./types";

import { api } from "./tauri";

const storage_key = "tokn.theme";

function readPreference(): ThemePreference {
  try {
    const value = localStorage.getItem(storage_key);
    if (value === "light" || value === "dark") return value;
  } catch {
    // Appearance still works when local storage is unavailable.
  }
  return "system";
}

export function useTheme() {
  const [preference, setPreference] = useState<ThemePreference>(readPreference);

  useLayoutEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      document.documentElement.dataset.theme =
        preference === "system"
          ? media.matches
            ? "dark"
            : "light"
          : preference;
    };
    apply();
    media.addEventListener("change", apply);
    try {
      localStorage.setItem(storage_key, preference);
    } catch {
      // Keep the selected appearance for this session.
    }
    return () => media.removeEventListener("change", apply);
  }, [preference]);

  useEffect(() => {
    void api
      .setTheme(preference === "system" ? null : preference)
      .catch((error) => {
        console.error("Unable to update native window appearance", error);
      });
  }, [preference]);

  return { preference, setPreference };
}
