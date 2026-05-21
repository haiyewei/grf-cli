# grf-cli

[English](./README.md) | 简体中文

`grf-cli` 正在重构为 Rust 原生 CLI。Node 不再承载运行时业务逻辑，npm
包只负责安装、升级和启动本地原生二进制。

## 当前状态

- 旧的 TypeScript 实现已经移除。
- Cargo 工程已经成为主运行时入口。
- 当前 Rust CLI 已具备这些命令：
  - `grf add`
  - `grf clean`
  - `grf list`
  - `grf load`
  - `grf unload`
  - `grf update`
- 旧的 `config` 命令已经移除。
- Rust 重写版本不再自动兼容或迁移旧版磁盘状态格式。
- 仓库已经具备正式发布闭环：
  - Rust 多平台构建
  - 当前 GitHub 仓库 Release 资产上传
  - npm 安装器包发布

## 安装

### 全局安装

```bash
pnpm add -g grf-cli
```

### 升级并更新可执行文件

```bash
pnpm update -g grf-cli
```

安装阶段的 `postinstall` 会按当前 npm 包版本下载当前 GitHub 仓库对应
release tag 下的原生二进制，并原地替换旧版本可执行文件。

## 本地开发

```bash
pnpm install --ignore-scripts
cargo build --release
cargo run -- --help
pnpm run check
```

当安装器检测到当前目录是仓库工作区时，npm 启动器会优先使用
`target/release/` 或 `target/debug/` 中本地构建出来的 Rust 二进制。

## 发布流程

正式版本工作流位于
[`publish.yml`](./.github/workflows/publish.yml)。

触发方式：

```text
push tag: vX.Y.Z
```

流水线步骤：

1. 校验 `package.json` 和 `Cargo.toml` 的版本一致。
2. 为配置的平台目标构建 `grf`。
3. 发布到当前 GitHub 仓库的 release。
4. 上传原生二进制作为 release 资产。
5. 发布 npm 安装器包。

工作流需要的仓库密钥：

- `NPM_TOKEN`

Release 上传使用当前仓库内建的 `GITHUB_TOKEN`。

## npm 安装器职责

发布后的 npm 包只保留三件事：

- 为 `grf`、`git-rf`、`gitref` 创建命令入口
- 在 `postinstall` 时下载匹配版本的原生二进制
- 在 `npm update` / `pnpm update` 后再次替换这个二进制

如果在正式发布前需要测试不同版本号下载，可以使用这些环境变量覆盖：

- `GRF_RELEASE_TAG`
- `GRF_INSTALL_LINUX_LIBC`（`gnu` 或 `musl`）

## License

MIT
