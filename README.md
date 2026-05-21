# grf-cli

English | [简体中文](./README.zh-CN.md)

`grf-cli` is being rebuilt as a Rust-native CLI. Node is no longer used for
runtime business logic. The npm package only handles installation, upgrades,
and launching the native binary.

## Current status

- The previous TypeScript implementation has been removed.
- The Cargo project is now initialized as the main runtime entry.
- The current Rust CLI surface includes:
  - `grf add`
  - `grf clean`
  - `grf list`
  - `grf load`
  - `grf unload`
  - `grf update`
- The legacy `config` command has been removed.
- The Rust rewrite no longer attempts to migrate or preserve old on-disk state formats automatically.
- The repository is already wired for formal release publishing:
  - Rust multi-platform builds
  - current GitHub repository release asset upload
  - npm wrapper package publish

## Install

### Global install

```bash
pnpm add -g grf-cli
```

### Update the installed binary

```bash
pnpm update -g grf-cli
```

`postinstall` resolves the package version, downloads the matching asset from
the current GitHub repository release tag, and replaces the previous native
binary in place.

## Local development

```bash
pnpm install --ignore-scripts
cargo build --release
cargo run -- --help
pnpm run check
```

The npm launcher will prefer the locally built Rust binary from
`target/release/` or `target/debug/` when the repository checkout is detected.

## Release flow

The formal release workflow lives in
[`publish.yml`](./.github/workflows/publish.yml).

Trigger:

```text
push tag: vX.Y.Z
```

Pipeline:

1. Verify `package.json` and `Cargo.toml` share the same version.
2. Build `grf` for the configured targets.
3. Publish binaries to the current GitHub repository release.
4. Upload native binaries as release assets.
5. Publish the npm installer package.

Repository secrets required by the workflow:

- `NPM_TOKEN`

The release upload uses the current repository's built-in `GITHUB_TOKEN`.

## npm wrapper behavior

The published npm package keeps only three responsibilities:

- create command shims for `grf`, `git-rf`, and `gitref`
- download the matching native executable during `postinstall`
- replace that executable again when the package is upgraded

If you need to test against a different release tag before publishing, the
installer supports these environment overrides:

- `GRF_RELEASE_TAG`
- `GRF_INSTALL_LINUX_LIBC` (`gnu` or `musl`)

## License

MIT
