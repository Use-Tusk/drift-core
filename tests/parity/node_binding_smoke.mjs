import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");

const fixturePath = path.join(repoRoot, "tests", "fixtures", "basic.json");
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf-8"));
const payloadJson = JSON.stringify(fixture.input);

const bindingModulePath = path.join(repoRoot, "bindings", "node", "index.js");

let binding;
try {
  // index.js handles loading built addon and throws if missing.
  binding = await import(bindingModulePath);
} catch (err) {
  throw new Error(
    `Node binding is not loadable. Build it first in bindings/node. Original error: ${String(err)}`,
  );
}

const normalized = binding.normalizeJson(payloadJson);
const digest = binding.deterministicHash(payloadJson);
const combined = binding.normalizeAndHash(payloadJson);
const protoBytes = binding.objectToProtobufStructBytes(payloadJson);
const fieldCount = binding.objectToProtobufStructFieldCount(payloadJson);

if (typeof normalized !== "string" || normalized.length === 0) {
  throw new Error("normalize_json did not return a non-empty string");
}
if (typeof digest !== "string" || digest.length !== 64) {
  throw new Error("deterministic_hash did not return a 64-char hex string");
}
if (
  !combined ||
  typeof combined.normalizedJson !== "string" ||
  typeof combined.deterministicHash !== "string"
) {
  throw new Error("normalizeAndHash did not return expected object shape");
}
if (combined.normalizedJson !== normalized || combined.deterministicHash !== digest) {
  throw new Error("normalizeAndHash result did not match individual API calls");
}
if (!Buffer.isBuffer(protoBytes) || protoBytes.length === 0) {
  throw new Error("object_to_protobuf_struct_bytes did not return non-empty Buffer");
}
if (!Number.isInteger(fieldCount) || fieldCount <= 0) {
  throw new Error("object_to_protobuf_struct_field_count did not return positive integer");
}

console.log("OK: node binding smoke passed");
