#!/usr/bin/env node
const { spawn } = require("node:child_process");
const { install } = require("../scripts/install-logic");

async function main() {
  try {
    const { kuracExe } = await install();
    const child = spawn(kuracExe, process.argv.slice(2), { stdio: "inherit" });
    child.on("exit", (code) => process.exit(code ?? 0));
  } catch (err) {
    console.error(`Failed to run kurac: ${err.message}`);
    process.exit(1);
  }
}
main();
