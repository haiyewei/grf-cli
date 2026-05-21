"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const {
  getBinaryConfig,
  getInstalledBinaryPath,
  getLocalBinaryCandidates,
  getPackageRoot,
  isRepositoryCheckout,
  loadPackageJson,
} = require("./shared.cjs");

function main() {
  const packageRoot = getPackageRoot();
  const packageJson = loadPackageJson(packageRoot);
  const { binaryName } = getBinaryConfig(packageJson);
  const installedBinary = getInstalledBinaryPath(packageRoot, binaryName);
  const localCandidates = getLocalBinaryCandidates(packageRoot, binaryName);

  const binaryPath = resolveBinaryPath(
    packageRoot,
    installedBinary,
    localCandidates,
  );

  const result = spawnSync(binaryPath, process.argv.slice(2), {
    cwd: process.cwd(),
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  if (typeof result.status === "number") {
    process.exit(result.status);
  }

  process.exit(1);
}

function resolveBinaryPath(packageRoot, installedBinary, localCandidates) {
  if (fs.existsSync(installedBinary)) {
    return installedBinary;
  }

  for (const candidate of localCandidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  if (isRepositoryCheckout(packageRoot)) {
    const relativeCandidates = localCandidates.map((candidate) =>
      path.relative(packageRoot, candidate),
    );
    throw new Error(
      [
        "未找到可执行文件。",
        "当前是仓库工作区，请先运行 `cargo build --release` 或 `pnpm run build`。",
        `已检查: ${relativeCandidates.join(", ")}`,
      ].join(" "),
    );
  }

  throw new Error(
    "未找到已安装的 grf 原生二进制。请重新执行包管理器安装，或运行 npm/pnpm update 触发 postinstall。",
  );
}

main();
