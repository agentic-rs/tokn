import { api } from "../lib/tauri";
import { inspectQuery } from "../lib/inspect-query";

export class InspectorError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "InspectorError";
  }
}

export async function readInspector<T>(
  locator: string,
  signal?: AbortSignal,
): Promise<T> {
  signal?.throwIfAborted();
  // Native database work is bounded but not cancellable mid-query. Stop waiting
  // immediately on navigation and discard late completion instead of updating
  // a newer selection with an old result.
  return new Promise<T>((resolve, reject) => {
    const abort = () =>
      reject(new DOMException("Inspection cancelled", "AbortError"));
    signal?.addEventListener("abort", abort, { once: true });
    void Promise.resolve()
      .then(() => api.inspect<T>(inspectQuery(locator)))
      .then(resolve, (error) => {
        if (
          typeof error === "object" &&
          error !== null &&
          "status" in error &&
          "message" in error
        ) {
          reject(
            new InspectorError(Number(error.status), String(error.message)),
          );
        } else {
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      })
      .finally(() => signal?.removeEventListener("abort", abort));
  });
}

export function isAbortError(error: unknown): boolean {
  return error instanceof Error && error.name === "AbortError";
}
