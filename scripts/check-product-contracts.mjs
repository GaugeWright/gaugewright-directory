import { createHash } from "node:crypto";
import { access, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const manifest = JSON.parse(await readFile(resolve(root, "contracts/product-routes.json"), "utf8"));
const source = await readFile(resolve(root, "src/lib.rs"), "utf8");
const failures = [];
const evidenceClasses = ["contract", "authority", "journey", "deployed", "property"];
const requiredFields = [
    "id", "jurisdiction", "transport", "method", "path", "producer", "consumer",
    "authentication", "scope", "capability", "requestSchema", "responseSchema",
    "sideEffect", "risk", "compatibility", "evidence",
];

if (manifest.schemaVersion !== 1) failures.push("manifest schemaVersion must be 1");
if (manifest.owner !== "gaugewright-directory") {
    failures.push("manifest owner must be gaugewright-directory");
}
if (!Array.isArray(manifest.contracts)) failures.push("manifest contracts must be an array");
if (!Array.isArray(manifest.exceptions)) failures.push("manifest exceptions must be an array");

const operations = new Set();
const ids = new Set();
for (const [index, contract] of (manifest.contracts ?? []).entries()) {
    const label = contract.id || `contract ${index + 1}`;
    for (const field of requiredFields) {
        if (!(field in contract) || contract[field] === "") failures.push(`${label} lacks ${field}`);
    }
    if (ids.has(contract.id)) failures.push(`duplicate contract id ${contract.id}`);
    ids.add(contract.id);
    const operation = `${contract.method} ${contract.path}`;
    if (operations.has(operation)) failures.push(`duplicate operation ${operation}`);
    operations.add(operation);
    if (!Number.isInteger(contract.compatibility?.contractVersion)) {
        failures.push(`${label} lacks an integer contract version`);
    }
    for (const evidence of evidenceClasses) {
        if (!Array.isArray(contract.evidence?.[evidence])) {
            failures.push(`${label} evidence.${evidence} must be an array`);
            continue;
        }
        for (const evidenceId of contract.evidence[evidence]) {
            const [relativePath] = evidenceId.split("#", 1);
            try {
                await access(resolve(root, relativePath));
            } catch {
                failures.push(`${label} evidence.${evidence} references missing ${relativePath}`);
            }
        }
    }
}

const registered = new Set();
for (const match of source.matchAll(
    /\.route\(\s*"([^"]+)"\s*,\s*put\([^)]*\)\.get\([^)]*\)\s*\)/gs,
)) {
    registered.add(`PUT ${match[1]}`);
    registered.add(`GET ${match[1]}`);
}
for (const operation of registered) {
    if (!operations.has(operation)) failures.push(`source operation ${operation} is undeclared`);
}
for (const operation of operations) {
    if (!registered.has(operation)) failures.push(`declared operation ${operation} is not registered`);
}

for (const exception of manifest.exceptions ?? []) {
    if (!exception.path || !exception.owner || !exception.reason) {
        failures.push("exception lacks path, owner, or reason");
    }
    if (!/^\d{4}-\d{2}-\d{2}$/.test(exception.expires ?? "")) {
        failures.push(`exception ${exception.path ?? "<unknown>"} has invalid expiry`);
    }
}

const evidenceGaps = [];
for (const contract of manifest.contracts ?? []) {
    const required = contract.risk === "critical"
        ? ["contract", "authority", "journey", "deployed"]
        : contract.risk === "important"
            ? ["contract", "authority", "journey"]
            : ["contract", "authority"];
    for (const evidence of required) {
        if (contract.evidence[evidence].length === 0) {
            evidenceGaps.push(`${contract.id}:${evidence}`);
        }
    }
}

function canonical(value) {
    if (Array.isArray(value)) return value.map(canonical);
    if (value && typeof value === "object") {
        return Object.fromEntries(Object.keys(value).sort().map((key) => [key, canonical(value[key])]));
    }
    return value;
}
const digest = createHash("sha256")
    .update(JSON.stringify(canonical(manifest)))
    .digest("hex");

if (failures.length) {
    console.error("Product contract validation failed:\n");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
}
console.log(
    `Product contract manifest sha256:${digest}: ${manifest.contracts.length} operations, `
    + `${manifest.exceptions.length} time-bounded exceptions, ${evidenceGaps.length} evidence gaps.`,
);
if (evidenceGaps.length) console.log(`Evidence gaps: ${evidenceGaps.join(", ")}`);
if (process.argv.includes("--enforce-evidence") && evidenceGaps.length) process.exit(1);
