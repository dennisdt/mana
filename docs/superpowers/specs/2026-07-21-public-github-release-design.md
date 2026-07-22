# Public GitHub Release Design

## Goal

Publish Mana as an open-source project at `https://github.com/dennisdt/mana` with source code, an MIT license, clear documentation, and a downloadable v0.4.5 macOS release.

## Repository

- Create `dennisdt/mana` as a public GitHub repository.
- Publish the existing `main` history without rewriting commits.
- Add `origin` pointing to the new repository and push `main` with upstream tracking.
- Ignore `.DS_Store`; do not publish local build output, dependencies, scratch files, credentials, or packaged binaries in Git history.
- Retain the existing source assets and engineering design/plan documents.

## Open-Source License

Add the standard MIT License with copyright `2026 Dennis Tran`.

## README

Rewrite the README for users and contributors. It will include:

1. Mana name, concise product description, and a screenshot of the current native widget.
2. Supported providers and the behavior that hides unauthenticated providers.
3. Requirements: macOS on Apple Silicon for the provided v0.4.5 binary, plus authenticated Claude Code and/or Codex CLI sessions.
4. Installation from the GitHub Release ZIP, including the first-launch Gatekeeper step required by the ad-hoc signed build.
5. An explicit security and privacy section describing local credential sources, read-only credential access, polling endpoints, local process checks, and the fact that credentials are never logged, persisted, refreshed, or written.
6. Source build, development, and test commands.
7. Known limitations: undocumented provider endpoints, potential breakage, Apple Silicon-only binary, and lack of Apple notarization.
8. Contribution guidance and MIT license reference.

The README must distinguish the downloadable binary from source compatibility. It must not imply that the app is signed, notarized, universal, or officially affiliated with Anthropic, OpenAI, or MapleStory.

## Screenshot

Capture the installed v0.4.5 native widget through CuaDriver without foregrounding another application. Store an optimized PNG at `docs/images/mana-widget.png` and reference it from the README. The screenshot must show the current production layout and contain no credentials or other private desktop content.

## Release

- Tag the reviewed source commit as `v0.4.5`.
- Create a non-draft public GitHub Release titled `Mana v0.4.5`.
- Attach `Mana-0.4.5-macOS-arm64.zip` built from the verified release bundle.
- Release notes will identify Apple Silicon support, ad-hoc signing, installation steps, provider visibility behavior, and the SHA-256 checksum.
- Verify the uploaded release URL, asset name, size, and checksum after publication.

## Validation

Before publishing:

- Run the frontend tests, Rust tests, and production macOS bundle build.
- Scan tracked files and history for credential-like material; test fixtures may contain clearly synthetic values only.
- Confirm the release archive contains `Mana.app`, reports version 0.4.5, targets `arm64`, and passes archive integrity testing.
- Review the final README as rendered Markdown and confirm all local links resolve.

After publishing:

- Confirm the GitHub repository is public and `main` is its default branch.
- Confirm the MIT license is detected or visibly present.
- Confirm the v0.4.5 release and downloadable archive are public.
