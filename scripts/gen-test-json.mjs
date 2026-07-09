#!/usr/bin/env node
// Generate a large JSON array of records for testing.
// Usage: node scripts/gen-test-json.mjs <count> <out.json>
// Example: node scripts/gen-test-json.mjs 2000000 /tmp/big.json  (~300 MB)
import { createWriteStream } from "node:fs";

const count = parseInt(process.argv[2] ?? "2000000", 10);
const out = process.argv[3] ?? "big.json";

const firsts = ["Alice", "Bob", "Carol", "Dan", "Eve", "Faythe", "Grace", "Heidi", "Ivan", "Judy"];
const cities = ["Beijing", "Shanghai", "New York", "Tokyo", "Paris", "Berlin", "Cairo", "São Paulo"];
const tagsPool = ["alpha", "beta", "gamma", "delta", "prod", "staging", "urgent", "review", "😀", "wörld"];

const ws = createWriteStream(out);
function write(s) {
  return new Promise((res) => {
    if (!ws.write(s)) ws.once("drain", res);
    else res();
  });
}

let seed = 12345;
const rnd = () => ((seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);
const pick = (a) => a[Math.floor(rnd() * a.length)];

console.log(`Generating ${count.toLocaleString()} records → ${out}`);
await write("[\n");
for (let i = 0; i < count; i++) {
  const rec = {
    id: i,
    uuid: `u-${(i * 2654435761 >>> 0).toString(16)}`,
    name: `${pick(firsts)} #${i}`,
    age: 18 + Math.floor(rnd() * 60),
    active: rnd() > 0.5,
    score: Math.round(rnd() * 100000) / 100,
    city: pick(cities),
    tags: [pick(tagsPool), pick(tagsPool)],
    meta: { created: 1_600_000_000 + i, note: rnd() > 0.9 ? null : "ok" },
  };
  await write(JSON.stringify(rec) + (i < count - 1 ? ",\n" : "\n"));
  if (i % 500000 === 0 && i) console.log(`  ${i.toLocaleString()}…`);
}
await write("]\n");
ws.end();
await new Promise((res) => ws.on("finish", res));
console.log("done");
