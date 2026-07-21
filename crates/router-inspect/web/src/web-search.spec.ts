import {
  decodedBase64ByteLength,
  inspectWebSearch,
  isCodexWebSearchEndpoint,
  safeHttpUrl
} from "./web-search.js";

function assertEqual(actual: unknown, expected: unknown, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}\nexpected ${JSON.stringify(expected)}\nreceived ${JSON.stringify(actual)}`);
  }
}

const inspection = inspectWebSearch(
  {
    input: [{ role: "user", content: [{ type: "input_text", text: "Original prompt" }] }],
    commands: {
      search_query: [{ q: "rust sqlite viewer", domains: ["docs.rs"], recency: 30 }],
      response_length: "short"
    },
    settings: { allowed_callers: ["direct"], external_web_access: false }
  },
  {
    output: "A compact synthesis",
    encrypted_output: "AQIDBA==",
    results: [
      {
        type: "text_result",
        domain: "docs.rs",
        ref_id: "turn1search0",
        title: "A result",
        url: "https://docs.rs/example",
        snippet: "Result snippet"
      }
    ]
  }
);

assertEqual(inspection, {
  queries: [{ query: "rust sqlite viewer", domains: ["docs.rs"], recency_days: 30 }],
  response_length: "short",
  allowed_callers: ["direct"],
  external_web_access: false,
  prompt: "Original prompt",
  output: "A compact synthesis",
  results: [{
    type: "text_result",
    domain: "docs.rs",
    ref_id: "turn1search0",
    snippet: "Result snippet",
    title: "A result",
    url: "https://docs.rs/example"
  }],
  encrypted_output_bytes: 4
}, "search payloads should become a compact inspection");

assertEqual(decodedBase64ByteLength("AQID"), 3, "unpadded base64 should have a decoded size");
assertEqual(decodedBase64ByteLength("gA-_"), 3, "URL-safe base64 should have a decoded size");
assertEqual(decodedBase64ByteLength("not base64!"), undefined, "invalid base64 should not have a decoded size");
assertEqual(
  isCodexWebSearchEndpoint("https://chatgpt.com/backend-api/codex/alpha/search?feature=1"),
  true,
  "full search URLs should match"
);
assertEqual(isCodexWebSearchEndpoint("/backend-api/codex/responses"), false, "other Codex URLs should not match");
assertEqual(safeHttpUrl("https://example.com/result"), "https://example.com/result", "HTTPS result URLs should be links");
assertEqual(safeHttpUrl("javascript:alert(1)"), undefined, "unsafe result URLs should not be links");

console.log("web-search tests passed");
