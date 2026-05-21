"use strict";

const fs = require("node:fs");
const path = require("node:path");
const {
  getBinaryConfig,
  getInstalledBinaryPath,
  getPackageRoot,
  isRepositoryCheckout,
  loadPackageJson,
} = require("./shared.cjs");

async function main() {
  const packageRoot = getPackageRoot();

  if (process.env.GRF_SKIP_POSTINSTALL === "1") {
    console.log("[grf] 跳过 postinstall：GRF_SKIP_POSTINSTALL=1");
    return;
  }

  if (isRepositoryCheckout(packageRoot)) {
    console.log("[grf] 检测到仓库工作区，跳过远程二进制下载。");
    return;
  }

  const packageJson = loadPackageJson(packageRoot);
  const config = getBinaryConfig(packageJson);
  validateReleaseConfig(config);

  const assetName = resolveAssetName(config.binaryName);
  const downloadUrl = resolveDownloadUrl(config, assetName);

  const installPath = getInstalledBinaryPath(packageRoot, config.binaryName);
  const installDir = path.dirname(installPath);
  const tempPath = `${installPath}.tmp`;

  fs.mkdirSync(installDir, { recursive: true });

  await downloadAsset(downloadUrl, tempPath);

  if (fs.existsSync(installPath)) {
    fs.rmSync(installPath, { force: true });
  }

  fs.renameSync(tempPath, installPath);

  if (process.platform !== "win32") {
    fs.chmodSync(installPath, 0o755);
  }

  console.log(`[grf] 已安装原生二进制: ${installPath}`);
}

function validateReleaseConfig(config) {
  if (!config.owner || !config.repo) {
    throw new Error(
      "缺少当前仓库的 GitHub release 配置。请检查 package.json 中的 repository.url。",
    );
  }
}

function resolveDownloadUrl(config, assetName) {
  return `https://github.com/${encodeURIComponent(config.owner)}/${encodeURIComponent(config.repo)}/releases/download/${encodeURIComponent(config.tag)}/${encodeURIComponent(assetName)}`;
}

function resolveAssetName(binaryName) {
  const arch = resolveArch();

  if (process.platform === "win32") {
    return `${binaryName}-${arch}-pc-windows-msvc.exe`;
  }

  if (process.platform === "darwin") {
    return `${binaryName}-${arch}-apple-darwin`;
  }

  if (process.platform === "linux") {
    return `${binaryName}-${arch}-unknown-linux-${resolveLinuxLibc()}`;
  }

  throw new Error(`当前 npm 安装器不支持平台: ${process.platform}`);
}

function resolveArch() {
  switch (process.arch) {
    case "x64":
      return "x86_64";
    case "arm64":
      return "aarch64";
    default:
      throw new Error(`当前 npm 安装器不支持架构: ${process.arch}`);
  }
}

function resolveLinuxLibc() {
  const override = process.env.GRF_INSTALL_LINUX_LIBC;

  if (override === "gnu" || override === "musl") {
    return override;
  }

  if (override) {
    throw new Error(
      `GRF_INSTALL_LINUX_LIBC 只支持 gnu 或 musl，收到: ${override}`,
    );
  }

  try {
    const report = process.report?.getReport?.();
    const runtime = report?.header?.glibcVersionRuntime;

    if (runtime) {
      return compareSemverLike(runtime, "2.35") >= 0 ? "gnu" : "musl";
    }
  } catch {
    // Ignore diagnostics failures and fall through to a safe default.
  }

  return "musl";
}

function compareSemverLike(left, right) {
  const leftParts = left.split(".").map((value) => Number.parseInt(value, 10));
  const rightParts = right
    .split(".")
    .map((value) => Number.parseInt(value, 10));
  const maxLength = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < maxLength; index += 1) {
    const leftValue = leftParts[index] ?? 0;
    const rightValue = rightParts[index] ?? 0;

    if (leftValue > rightValue) {
      return 1;
    }

    if (leftValue < rightValue) {
      return -1;
    }
  }

  return 0;
}

async function downloadAsset(url, outputPath) {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(
      `下载当前仓库 release 二进制失败 (${response.status}): ${url}`,
    );
  }

  const data = Buffer.from(await response.arrayBuffer());
  fs.writeFileSync(outputPath, data);
}

main().catch((error) => {
  console.error(`[grf] ${error.message}`);
  process.exit(1);
});
