# Release checklist

The GitHub repository and release automation are already connected. A pushed
version tag triggers CI builds for Windows, macOS, and Linux and publishes the
installers to the matching GitHub Release.

## Before tagging

1. Update the version in both `Cargo.toml` and `tauri.conf.json`.
2. Confirm the working tree contains only the intended release changes:
   ```bash
   git status --short
   ```
3. Run the verification suite:
   ```bash
   cargo build
   cargo test
   cargo tauri build
   ```
4. Commit the release changes to `main`.

## Publish and verify

```bash
TAG=vX.Y.Z
git tag "$TAG"
git push origin main
git push origin "$TAG"

# Watch the matching tag workflow, then confirm the release and installers.
RUN_ID=$(gh run list --workflow build.yml --branch "$TAG" --event push --limit 1 \
  --json databaseId --jq '.[0].databaseId')
test -n "$RUN_ID"
gh run watch "$RUN_ID" --exit-status
gh release view "$TAG"
```

Do not claim a release is shipped until the tag workflow succeeds and
`gh release view vX.Y.Z` shows the release assets.
