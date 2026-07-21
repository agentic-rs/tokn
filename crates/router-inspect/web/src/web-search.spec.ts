import {
  decodedBase64ByteLength,
  inspectWebSearch,
  isCodexWebSearchEndpoint,
  safeHttpUrl,
  webSearchOperationSummary
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
  operations: [{ kind: "search_query", value: "rust sqlite viewer", domains: ["docs.rs"], recency_days: 30 }],
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

const open_inspection = inspectWebSearch({
  commands: {
    open: [
      { ref_id: "https://example.com/one" },
      { ref_id: "turn1view0", lineno: 42 }
    ],
    response_length: "long"
  }
}, {});
assertEqual(open_inspection.operations, [
  { kind: "open", value: "https://example.com/one" },
  { kind: "open", value: "turn1view0", line_number: 42 }
], "open commands should be represented as web operations");
assertEqual(webSearchOperationSummary(open_inspection.operations), "2 page opens", "open requests need an accurate heading");
assertEqual(webSearchOperationSummary([]), "No operations", "missing commands should not be described as zero queries");

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
