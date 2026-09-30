#!/usr/bin/env node
// A fake `websign connect`: speaks the framed protocol from a scenario file.
//
// WEBSIGN_FAKE_SCENARIO points at JSON:
//   log?          file that receives every message this process reads (JSON lines)
//   pidFile?      file that receives this process's pid at startup
//   argvFile?     file that receives process.argv.slice(2) as JSON
//   ignoreStdinEnd?  keep running after stdin closes (to exercise the kill path)
//   rules         [{ on: "start" | <message type>, where?: {field: value}, repeat?, do: [action] }]
// Each rule fires on the first matching message and is then spent unless `repeat`.
// Actions: send {id?, ...}, sendMany [..], sendSplit {message, chunk}, sendRaw base64,
//          header n, delay ms, exit code.
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";

const scenario = JSON.parse(readFileSync(process.env.WEBSIGN_FAKE_SCENARIO, "utf8"));
if (scenario.pidFile) writeFileSync(scenario.pidFile, String(process.pid));
if (scenario.argvFile) writeFileSync(scenario.argvFile, JSON.stringify(process.argv.slice(2)));

const frame = (message) => {
  const body = Buffer.from(JSON.stringify(message));
  const header = Buffer.alloc(4);
  header.writeUInt32LE(body.length);
  return Buffer.concat([header, body]);
};
const envelope = (message, id) => ({ v: 1, id, ...message });
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const write = (bytes) => new Promise((resolve) => process.stdout.write(bytes, resolve));

async function perform(action, trigger) {
  const id = trigger?.id ?? "n0";
  if ("send" in action) await write(frame(envelope(action.send, id)));
  else if ("sendMany" in action)
    await write(Buffer.concat(action.sendMany.map((m) => frame(envelope(m, id)))));
  else if ("sendSplit" in action) {
    const bytes = frame(envelope(action.sendSplit.message, id));
    for (let i = 0; i < bytes.length; i += action.sendSplit.chunk) {
      await write(bytes.subarray(i, i + action.sendSplit.chunk));
      await sleep(2);
    }
  } else if ("sendRaw" in action) await write(Buffer.from(action.sendRaw, "base64"));
  else if ("header" in action) {
    const header = Buffer.alloc(4);
    header.writeUInt32LE(action.header);
    await write(header);
  } else if ("delay" in action) await sleep(action.delay);
  else if ("exit" in action) process.exit(action.exit);
}

const matches = (rule, message) =>
  rule.on === message.type &&
  Object.entries(rule.where ?? {}).every(([key, value]) => message[key] === value);

async function handle(message) {
  if (scenario.log) appendFileSync(scenario.log, `${JSON.stringify(message)}\n`);
  const rule = scenario.rules.find((r) => !r.spent && matches(r, message));
  if (!rule) return;
  rule.spent = !rule.repeat;
  for (const action of rule.do) await perform(action, message);
}

let queue = Promise.resolve();
let buffered = Buffer.alloc(0);
process.stdin.on("data", (chunk) => {
  buffered = Buffer.concat([buffered, chunk]);
  while (buffered.length >= 4 && buffered.length >= 4 + buffered.readUInt32LE(0)) {
    const length = buffered.readUInt32LE(0);
    const message = JSON.parse(buffered.subarray(4, 4 + length).toString("utf8"));
    buffered = buffered.subarray(4 + length);
    queue = queue.then(() => handle(message));
  }
});
process.stdin.on("end", () => {
  if (!scenario.ignoreStdinEnd) queue.then(() => process.exit(0));
});
process.on("SIGTERM", () => {});

for (const rule of scenario.rules.filter((r) => r.on === "start")) {
  rule.spent = true;
  queue = queue.then(async () => {
    for (const action of rule.do) await perform(action, undefined);
  });
}
if (scenario.ignoreStdinEnd) setInterval(() => {}, 1000);
else process.stdin.resume();
