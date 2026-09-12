#!/usr/bin/env python3
"""Build the Phase 15 contact sheets from scripts/check-v2-visual.mjs output.

  docs/design/v2/visual-contact-sheet.html   every capture, linked from .cache/v2-visual
                                             (run the capture script first; nothing heavy
                                             is committed)
  docs/design/v2/screenshots/phase15/*.jpg    the curated approval set, committed
  <out>/visual-approval.html                  the approval set as one self-contained file
                                             (--approval-out), for sharing
"""
from __future__ import annotations

import argparse
import base64
import html
import io
import json
import re
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / ".cache" / "v2-visual"
V2 = ROOT / "docs" / "design" / "v2"
APPROVAL_DIR = V2 / "screenshots" / "phase15"
SCREENS = ["overview", "discover", "my-apps", "activity", "settings", "install", "recovery", "first-run", "panel"]
LABELS = {"overflow": "Horizontal overflow", "truncation": "Unreadable truncation", "glyphClip": "Clipped glyphs",
          "overlap": "Overlap or spilled logo", "obscured": "Covered primary", "dialogs": "Unfit dialogs",
          "determinism": "Non-deterministic captures", "states": "Indistinct states",
          "commitBelowFold": "Focused commit off screen at 1280×640"}
FOLDERS = {"laptop-1280x720@1.5x": "1920×1080 laptop at 150% (1280×720 CSS px)",
           "scaled-1280x800@1.5x": "1280×800 at 150% display scaling"}
SHORT_KEYS = ["install-failure", "install-fail-port_in_use", "recovery-success-keep", "recovery-success-delete",
              "recovery-mismatch", "recovery-scan-failure", "first-run-welcome-missing", "discover-detail"]
STRESS_KEYS = ["my-apps-default", "my-apps-linked", "my-apps-error-row", "discover-default", "install-default",
               "install-evidence", "install-fail-port_in_use", "overview-default", "my-apps-delete-data"]

STYLE = """
:root{color-scheme:dark;--bg:#0c0a09;--panel:#161211;--line:rgb(255 247 239/9%);--ink:#f7f1e9;--body:#c3b8ae;--muted:#998b80;--ember:#f26419;--ok:#62c79a;--bad:#e8614c}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--body);font:15px/1.55 "Segoe UI",system-ui,sans-serif}
main{max-width:1500px;margin:0 auto;padding:40px 32px 80px}h1,h2,h3{color:var(--ink);margin:0}
h1{font-size:34px;letter-spacing:-.03em}h2{font-size:22px;margin:48px 0 6px}h3{font-size:14px;margin:26px 0 10px;color:var(--muted);text-transform:uppercase;letter-spacing:.08em;font-family:Consolas,monospace}
p{max-width:78ch}.lede{font-size:17px;margin:10px 0 0}
.stats{display:flex;flex-wrap:wrap;gap:10px;margin:24px 0 0}.stat{padding:12px 16px;border:1px solid var(--line);border-radius:12px;background:var(--panel)}
.stat b{display:block;color:var(--ink);font-size:20px;font-family:Consolas,monospace}.stat.pass b{color:var(--ok)}.stat.fail b{color:var(--bad)}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:14px}
figure{margin:0;border:1px solid var(--line);border-radius:12px;background:var(--panel);overflow:hidden}
figure img{display:block;width:100%;height:auto;background:#000}figcaption{padding:8px 10px;font:12px/1.4 Consolas,monospace;color:var(--muted);overflow-wrap:anywhere}
.states{grid-template-columns:repeat(4,minmax(0,1fr))}.states img{object-fit:contain;height:120px;padding:10px;background:var(--bg)}
table{border-collapse:collapse;width:100%;margin-top:12px}th,td{text-align:left;padding:8px 10px;border-bottom:1px solid var(--line);font-size:13.5px;vertical-align:top}th{color:var(--muted);font-weight:600}
code{font-family:Consolas,monospace;color:var(--ink)}a{color:#ff9b5c}
"""


def thumb(src: Path, width: int) -> bytes:
    with Image.open(src) as image:
        image = image.convert("RGB")
        if image.width > width:
            image = image.resize((width, round(image.height * width / image.width)), Image.LANCZOS)
        buffer = io.BytesIO()
        image.save(buffer, "JPEG", quality=74, optimize=True, progressive=True)
        return buffer.getvalue()


def captures(folder: str) -> list[Path]:
    path = CACHE / folder
    return sorted(path.glob("*.png")) if path.is_dir() else []


def screen_of(stem: str) -> str:
    return next((s for s in SCREENS if stem.startswith(s)), "other")


def summary_block(report: dict) -> str:
    findings = report["findings"]
    stats = [("States rendered", sum(report["passes"].get(k, 0) for k in report["passes"] if not k.startswith("determinism") and k != "interactionStates"), "pass")]
    stats += [(LABELS.get(k, k), len(v), "pass" if not v else "fail") for k, v in findings.items()]
    stats.append(("Captures re-taken for determinism", report["passes"].get("determinismSample", 0), "pass" if not findings["determinism"] else "fail"))
    stats.append(("Interaction-state checks", report["passes"].get("interactionStates", 0), "pass" if not findings["states"] else "fail"))
    cells = "".join(f'<div class="stat {cls}"><b>{value}</b>{html.escape(label)}</div>' for label, value, cls in stats)
    scaling = "".join(
        f"<tr><td>{html.escape(s['panel'])}</td><td><code>{s['cssViewport']}</code></td><td><code>{s['devicePixelRatio']}x</code></td>"
        f"<td>{html.escape(', '.join(f'{k}: {v}' for k, v in s['newFindings'].items()) or 'none')}</td></tr>"
        for s in report["review"]["scaling"])
    return (f'<div class="stats">{cells}</div>'
            f"<h3>Windows display scaling</h3><table><tr><th>Laptop panel</th><th>CSS viewport</th><th>Pixel ratio</th><th>New findings</th></tr>{scaling}</table>")


def figure(src: str, caption: str, cls: str = "") -> str:
    return f'<figure class="{cls}"><a href="{src}"><img loading="lazy" src="{src}" alt="{html.escape(caption)}"></a><figcaption>{html.escape(caption)}</figcaption></figure>'


def build(report: dict, approval_out: Path | None) -> None:
    rel = "../../../.cache/v2-visual"
    parts = [f"<title>Local Store V2 visual verification</title><style>{STYLE}</style><main>",
             "<h1>Visual verification</h1><p class=\"lede\">Every prototype state at both laptop viewports, "
             "worst-case real content, display scaling, and interaction states. Generated by "
             "<code>scripts/check-v2-visual.mjs</code> and <code>scripts/build-v2-contact-sheet.py</code>.</p>",
             summary_block(report)]
    for folder, title in [("normal-1440x900", "1440 × 900 — primary"), ("normal-1280x800", "1280 × 800 — minimum"),
                           ("stress-1280x800", "Stress content at 1280 × 800"), ("stress-1440x900", "Stress content at 1440 × 900"),
                           ("short-1280x640@1.5x", "1280 × 640 — maximised 1920×1080 laptop at 150%")]:
        files = captures(folder)
        if not files:
            continue
        parts.append(f"<h2>{title}</h2>")
        for screen in SCREENS:
            group = [f for f in files if screen_of(f.stem) == screen]
            if group:
                parts.append(f'<h3>{screen}</h3><div class="grid">' + "".join(figure(f"{rel}/{folder}/{f.name}", f.stem) for f in group) + "</div>")
    scaled = [p for p in CACHE.glob("scaled-*") if p.is_dir()] + [p for p in CACHE.glob("laptop-*") if p.is_dir()]
    if scaled:
        parts.append("<h2>Display scaling</h2>")
        for folder in sorted(scaled):
            parts.append(f"<h3>{folder.name}</h3><div class=\"grid\">" + "".join(figure(f"{rel}/{folder.name}/{f.name}", f.stem) for f in captures(folder.name)) + "</div>")
    # Controls with rest/hover/active/focus first, four to a row; then single states.
    state_files = sorted(captures("states"), key=lambda f: (not re.search(r"-[1-4]-", f.stem), f.stem))
    if state_files:
        parts.append('<h2>Interaction states</h2><div class="grid states">' + "".join(figure(f"{rel}/states/{f.name}", f.stem, "") for f in state_files) + "</div>")
    parts.append("</main>")
    (V2 / "visual-contact-sheet.html").write_text("\n".join(parts), encoding="utf-8")

    # Curated approval set: every 1280 state, the stress highlights, scaling, states.
    APPROVAL_DIR.mkdir(parents=True, exist_ok=True)
    for old in APPROVAL_DIR.glob("*.jpg"):
        old.unlink()
    approval: list[tuple[str, str, bytes, str]] = []
    for f in captures("normal-1280x800"):
        approval.append(("1280 × 800", f.stem, thumb(f, 640), ""))
    for f in captures("stress-1280x800"):
        if f.stem in STRESS_KEYS:
            approval.append(("Stress content, 1280 × 800", f.stem, thumb(f, 640), ""))
    for f in captures("short-1280x640@1.5x"):
        if f.stem in SHORT_KEYS:
            approval.append(("Short window, 1280 × 640 at 150%", f.stem, thumb(f, 640), ""))
    for folder in ["laptop-1280x720@1.5x", "scaled-1280x800@1.5x"]:
        for f in captures(folder):
            if f.stem in {"overview-default", "discover-default", "my-apps-default", "install-default", "first-run-welcome-ready"}:
                approval.append((FOLDERS[folder], f.stem, thumb(f, 640), ""))
    for f in state_files:
        # File names carry the order: <control>-1-rest, -2-hover, -3-active, -4-focus.
        approval.append(("Interaction states", f.stem.replace("-1-", " · ").replace("-2-", " · ").replace("-3-", " · ").replace("-4-", " · "), thumb(f, 480), "states"))
    sections: dict[str, list[str]] = {}
    for group, stem, data, cls in approval:
        name = f"{group.split(',')[0].split(' (')[0].replace(' × ', 'x').replace('×', 'x').replace(' ', '-').replace('%', 'pct').lower()}-{stem.replace(' · ', '-')}.jpg"
        (APPROVAL_DIR / name).write_bytes(data)
        uri = "data:image/jpeg;base64," + base64.b64encode(data).decode("ascii")
        sections.setdefault(group, []).append(figure(uri, stem, cls))
    if approval_out:
        body = [f"<title>Local Store V2 visual approval</title><style>{STYLE}</style><main>",
                "<h1>Visual approval set</h1><p class=\"lede\">Phase 15 of the Local Store V2 design: every prototype state at the "
                "1280 × 800 minimum viewport, worst-case real content, the short 1280 × 640 window, the 150% display-scaling "
                "cases, and each control's interaction states.</p>", summary_block(report)]
        for group, figures in sections.items():
            cls = " states" if group == "Interaction states" else ""
            body.append(f"<h2>{html.escape(group)}</h2><div class=\"grid{cls}\">{''.join(figures)}</div>")
        body.append("</main>")
        approval_out.write_text("\n".join(body), encoding="utf-8")
    size = sum(p.stat().st_size for p in APPROVAL_DIR.glob("*.jpg"))
    print(f"contact sheet: {V2 / 'visual-contact-sheet.html'}")
    print(f"approval set: {len(approval)} images, {size / 1_048_576:.1f} MB in {APPROVAL_DIR.relative_to(ROOT)}")
    if approval_out:
        print(f"self-contained approval sheet: {approval_out} ({approval_out.stat().st_size / 1_048_576:.1f} MB)")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--approval-out", type=Path)
    args = parser.parse_args()
    build(json.loads((V2 / "visual-report.json").read_text(encoding="utf-8")), args.approval_out)
