import Ajv2020 from "ajv/dist/2020.js";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const repo = resolve(process.env.WEFT_REPO ?? resolve(import.meta.dirname, "../../../../.."));
const evidence = resolve(process.env.WEFT_EVIDENCE ?? import.meta.dirname);
const contracts = resolve(repo, "docs/helix/02-design/contracts");
const read = (name) => JSON.parse(readFileSync(resolve(contracts, name), "utf8"));
const ajv = new Ajv2020({ allErrors: true, strict: false });
// ordinary-page.schema.json still references the frozen v6 output definition.
ajv.addSchema(read("generated-read-algebra-v6.schema.json"));
ajv.addSchema(read("query-codec-instructions.schema.json"));
ajv.addSchema(read("generated-ordinary-page.schema.json"));
const validate = ajv.compile(read("generated-read-algebra-v7.schema.json"));
const artifact = JSON.parse(readFileSync(resolve(evidence, "issued-v7.json"), "utf8"));
if (!validate(artifact)) throw new Error(JSON.stringify(validate.errors));

const outputLocations = [
  ["Page field output", ["ordinaryPage", "page", "fieldOutputs", 0, "output"]],
  ["Page Key output", ["ordinaryPage", "key", "output"]],
  ["Page hidden Key output", ["ordinaryPage", "page", "hiddenKeyOutput"]],
  ["forward Key output", ["forwardConnection", "key", "output"]],
  ["forward execution Key output", ["forwardConnection", "forward", "keyOutput"]],
  ["forward Field output", ["forwardConnection", "fieldOutputs", 0, "output"]],
];
for (const [label, path] of outputLocations) {
  for (const mutation of ["omit-length", "wrong-oid"]) {
    const candidate = structuredClone(artifact);
    let cursor = candidate.roots["demo/Note"];
    for (const key of path.slice(0, -1)) cursor = cursor[key];
    const output = cursor[path.at(-1)];
    output.nativeType = mutation === "omit-length" ? { kind: "varchar" } : { kind: "varchar", length: 32 };
    output.oid = mutation === "wrong-oid" ? 25 : 1043;
    if (validate(candidate)) throw new Error(`accepted ${label} ${mutation}`);
  }
}
for (const [label, mutate] of [
  ["Page eligible / connection refused", (root) => {
    root.forwardConnectionSupport = { profile: "weft.generated-forward-connection/1", status: "unsupported", reason: "schema-vector" };
    delete root.forwardConnection;
  }],
  ["connection eligible / Page refused", (root) => {
    root.ordinaryPage = { profile: "weft.generated-ordinary-page/1", status: "unsupported", reason: "schema-vector" };
  }],
]) {
  const candidate = structuredClone(artifact);
  mutate(candidate.roots["demo/Note"]);
  if (!validate(candidate)) throw new Error(`${label}: ${JSON.stringify(validate.errors)}`);
}
console.log(`valid baseline; rejected ${outputLocations.length * 2} output mutations; accepted both independent refusal directions`);
