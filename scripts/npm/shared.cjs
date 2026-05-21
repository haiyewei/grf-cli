"use strict";

const fs = require("node:fs");
const path = require("node:path");

function getPackageRoot() {
  return path.resolve(__dirname, "..", "..");
}

function loadPackageJson(packageRoot = getPackageRoot()) {
  const packageJsonPath = path.join(packageRoot, "package.json");
  return JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
}

function getBinaryConfig(packageJson) {
  const repository = resolveGitHubRepository(packageJson);

  return {
    binaryName: packageJson.grf?.binaryName ?? "grf",
    owner: repository.owner,
    repo: repository.repo,
    tag: process.env.GRF_RELEASE_TAG ?? `v${packageJson.version}`,
  };
}

function resolveGitHubRepository(packageJson) {
  const repository =
    typeof packageJson.repository === "string"
      ? packageJson.repository
      : packageJson.repository?.url;

  if (!repository) {
    throw new Error("package.json 缺少 repository.url，无法定位当前仓库。");
  }

  const normalized = repository.replace(/^git\+/, "");
  const httpsMatch = normalized.match(
    /^https?:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?$/,
  );

  if (httpsMatch) {
    return {
      owner: httpsMatch[1],
      repo: httpsMatch[2],
    };
  }

  const sshMatch = normalized.match(
    /^git@github\.com:([^/]+)\/([^/]+?)(?:\.git)?$/,
  );
  if (sshMatch) {
    return {
      owner: sshMatch[1],
      repo: sshMatch[2],
    };
  }

  throw new Error(`repository.url 不是可识别的 GitHub 仓库地址: ${repository}`);
}

function getExecutableName(binaryName) {
  return process.platform === "win32" ? `${binaryName}.exe` : binaryName;
}

function getInstalledBinaryPath(packageRoot, binaryName) {
  return path.join(packageRoot, "vendor", "bin", getExecutableName(binaryName));
}

function getLocalBinaryCandidates(packageRoot, binaryName) {
  const executableName = getExecutableName(binaryName);

  return [
    path.join(packageRoot, "target", "release", executableName),
    path.join(packageRoot, "target", "debug", executableName),
  ];
}

function isRepositoryCheckout(packageRoot) {
  return fs.existsSync(path.join(packageRoot, ".git"));
}

function readCargoVersion(packageRoot = getPackageRoot()) {
  const cargoTomlPath = path.join(packageRoot, "Cargo.toml");
  const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
  const packageSectionMatch = cargoToml.match(
    /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
  );

  if (!packageSectionMatch) {
    throw new Error("无法从 Cargo.toml 读取 [package].version");
  }

  return packageSectionMatch[1];
}

module.exports = {
  getBinaryConfig,
  getExecutableName,
  getInstalledBinaryPath,
  getLocalBinaryCandidates,
  getPackageRoot,
  isRepositoryCheckout,
  loadPackageJson,
  readCargoVersion,
  resolveGitHubRepository,
};
