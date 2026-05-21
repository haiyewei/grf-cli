"use strict";

const {
  getPackageRoot,
  loadPackageJson,
  readCargoVersion,
} = require("./shared.cjs");

function main() {
  const packageRoot = getPackageRoot();
  const packageJson = loadPackageJson(packageRoot);
  const cargoVersion = readCargoVersion(packageRoot);
  const packageVersion = packageJson.version;

  if (packageVersion !== cargoVersion) {
    throw new Error(
      `package.json 版本 ${packageVersion} 与 Cargo.toml 版本 ${cargoVersion} 不一致。`,
    );
  }

  const refName = process.env.GRF_RELEASE_REF_NAME ?? process.env.GITHUB_REF_NAME;
  const expectedTag = `v${packageVersion}`;

  if (refName && refName !== expectedTag) {
    throw new Error(
      `发布 tag 必须与版本一致，期望 ${expectedTag}，收到 ${refName}。`,
    );
  }

  console.log(`[grf] release version verified: ${expectedTag}`);
}

main();
