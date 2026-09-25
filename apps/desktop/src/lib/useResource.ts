import { useCallback, useEffect, useRef, useState } from "react";
export function useResource<T>(load: () => Promise<T>) {
  const [data, setData] = useState<T>();
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const generation = useRef(0);
  const refresh = useCallback(async () => {
    const current = ++generation.current;
    setLoading(true);
    setError("");
    try {
      const result = await load();
      if (current === generation.current) setData(result);
    } catch (error) {
      if (current === generation.current) setError(String(error));
    } finally {
      if (current === generation.current) setLoading(false);
    }
  }, [load]);
  useEffect(() => {
    void refresh();
    return () => {
      generation.current++;
    };
  }, [refresh]);
  return { data, setData, error, loading, refresh };
}
