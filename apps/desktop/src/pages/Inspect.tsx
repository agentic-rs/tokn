import { useEffect, useRef } from "react";
import "../inspect/app";

export function Inspect() {
  const container = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = document.createElement("inspect-app");
    container.current?.append(element);
    return () => element.remove();
  }, []);
  return (
    <div
      className="inspector-container"
      ref={container}
      aria-label="Request and session inspector"
    />
  );
}
