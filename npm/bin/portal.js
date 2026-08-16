#!/usr/bin/env node
const { spawn } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

const override = process.env.PORTAL_NPM_TARGET;
const platform = process.platform;
const arch = process.arch;

let target = override;
if (!target) {
  if (platform === "win32" && arch === "x64") target = "x86_64-pc-windows-msvc";
  else if (platform === "darwin" && arch === "arm64") target = "aarch64-apple-darwin";
  else if (platform === "darwin" && arch === "x64") target = "x86_64-apple-darwin";
  else if (platform === "android" && arch === "arm64") target = "aarch64-linux-android";
  else if (platform === "linux" && arch === "arm64") target = "aarch64-unknown-linux-gnu";
  else if (platform === "linux" && arch === "x64") target = "x86_64-unknown-linux-gnu";
}

if (!target) {
  console.error("Unsupported platform:", platform, arch);
  process.exit(1);
}

const binName = platform === "win32" ? "portal.exe" : "portal";
const binPath = path.join(__dirname, target, binName);

if (!fs.existsSync(binPath)) {
  console.error("Binary not found for target:", target);
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code) => process.exit(code ?? 0));