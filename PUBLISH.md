# Publish checklist (run after connecting GitHub)

1. Install GitHub CLI: https://cli.github.com/
2. Authenticate:
       gh auth login
3. From this folder, create + push the repo:
       gh repo create dockwrap --public --source=. --push -m main
4. Tag v0.1.0 (triggers CI -> release installers on Windows/macOS/Linux):
       git tag v0.1.0
       git push origin v0.1.0
5. Check the Actions tab — when the build job finishes, download
   installers from the new GitHub Release.

Local build (no GitHub needed):
       cargo install tauri-cli --version "^2" --locked
       cargo tauri build
