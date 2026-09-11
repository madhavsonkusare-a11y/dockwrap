// Browser-side layout probe shared by the V2 visual verification scripts.
// Pass it to page.evaluate(). It reports horizontal overflow, text cut with no way
// to read it, glyph clipping, overlapping controls, covered or off-screen primary
// actions, and dialogs that do not fit or hide a commit / irreversible action.
export const probeLayout = () => {
  const visible = (el) => {
    const r = el.getBoundingClientRect(); const s = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && s.visibility !== "hidden" && s.display !== "none" && !el.closest("[hidden]");
  };
  const name = (el) => `${el.tagName.toLowerCase()}${[...el.classList].slice(0, 2).map((c) => "." + c).join("")}`;
  const text = (el) => el.textContent.trim().replace(/\s+/g, " ");
  const inertAncestor = (el) => Boolean(el.closest("[inert]"));
  const out = { overflowX: false, truncated: [], glyphClip: [], overlaps: [], obscured: [], dialogs: [], belowFold: [] };

  out.overflowX = document.documentElement.scrollWidth > window.innerWidth + 1;

  // Truncation: text cut by its own box. Recoverable only if the full value can
  // still be read - a title / aria-label, the same text shown whole elsewhere, or a
  // catalog card's Details control for clamped descriptions.
  const shownWhole = new Set();
  const cut = [];
  for (const el of document.querySelectorAll("body *")) {
    if (!visible(el) || el.closest('[aria-hidden="true"], .svg-sprite')) continue;
    if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const s = getComputedStyle(el);
    const clipX = el.scrollWidth > el.clientWidth + 1 && ["hidden", "clip"].includes(s.overflowX);
    const clipY = el.scrollHeight > el.clientHeight + 1 && ["hidden", "clip"].includes(s.overflowY);
    const clamp = clipY && s.webkitLineClamp !== "none" && s.webkitLineClamp !== "";
    // A tight line-height with overflow:hidden can shave glyph ascenders or
    // descenders without hiding any characters. Track it separately: it is a
    // rendering defect, not lost content.
    const glyph = !clipX && clipY && !clamp;
    if (clipX || clamp) cut.push({ el, clamp, overflowX: el.scrollWidth - el.clientWidth });
    else if (glyph) out.glyphClip.push({ where: name(el), text: text(el).slice(0, 50), px: el.scrollHeight - el.clientHeight, lineHeight: s.lineHeight, fontSize: s.fontSize });
    else shownWhole.add(text(el));
  }
  for (const { el, clamp, overflowX } of cut) {
    const full = text(el);
    const labelled = [el, el.parentElement, el.parentElement?.parentElement].some((n) => n && ((n.getAttribute("title") || "").includes(full) || (n.getAttribute("aria-label") || "").includes(full)));
    const details = clamp && el.closest(".catalog-card")?.querySelector("[data-open-project]");
    const elsewhere = [...shownWhole].some((t) => t.includes(full));
    out.truncated.push({ where: name(el), text: full.slice(0, 90), kind: clamp ? "clamp" : `ellipsis -${overflowX}px`, recoverable: Boolean(labelled || details || elsewhere) });
  }

  // Overlapping controls (ignoring nesting and anything behind an open modal). A
  // popover, or a sticky bar the page scrolls beneath, is layering rather than a
  // collision: only controls in the same layer are compared.
  const layer = (el) => {
    for (let n = el; n; n = n.parentElement) {
      if (n.matches('.filter-popover, [role="dialog"], [role="menu"], [role="listbox"]')) return n;
      if (["fixed", "sticky"].includes(getComputedStyle(n).position)) return n; // content scrolls beneath it
    }
    return null;
  };
  const controls = [...document.querySelectorAll('a[href], button, input, select, textarea, [role="option"], [role="tab"]')]
    .filter((el) => visible(el) && !inertAncestor(el) && getComputedStyle(el).pointerEvents !== "none");
  for (let i = 0; i < controls.length; i += 1) {
    const a = controls[i].getBoundingClientRect();
    for (let j = i + 1; j < controls.length; j += 1) {
      if (controls[i].contains(controls[j]) || controls[j].contains(controls[i])) continue;
      if (layer(controls[i]) !== layer(controls[j])) continue;
      const b = controls[j].getBoundingClientRect();
      const w = Math.min(a.right, b.right) - Math.max(a.left, b.left);
      const h = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
      if (w > 2 && h > 2) out.overlaps.push(`${name(controls[i])} "${text(controls[i]).slice(0, 24)}" x ${name(controls[j])} "${text(controls[j]).slice(0, 24)}"`);
    }
  }

  // Images must stay inside their frame; a spilled logo paints over neighbouring copy.
  for (const img of document.querySelectorAll(".app-icon img")) {
    if (!visible(img)) continue;
    const i = img.getBoundingClientRect(); const f = img.parentElement.getBoundingClientRect();
    const spill = Math.max(f.top - i.top, i.bottom - f.bottom, f.left - i.left, i.right - f.right);
    if (spill > 1) out.overlaps.push(`img in ${name(img.parentElement)} spills ${Math.round(spill)}px outside its ${Math.round(f.width)}x${Math.round(f.height)} frame (${img.getAttribute("src").split("/").pop()})`);
  }

  // Primary actions: covered, or below the fold.
  for (const button of document.querySelectorAll(".button.primary")) {
    if (!visible(button) || inertAncestor(button)) continue;
    const r = button.getBoundingClientRect();
    const cx = r.left + r.width / 2; const cy = r.top + r.height / 2;
    if (r.top >= window.innerHeight) { out.belowFold.push(`${text(button).slice(0, 40)} (top ${Math.round(r.top)})`); continue; }
    if (cy < 0 || cy > window.innerHeight || cx < 0 || cx > window.innerWidth) continue;
    const hit = document.elementFromPoint(cx, cy);
    if (hit && !button.contains(hit)) out.obscured.push(`${text(button).slice(0, 40)} under ${name(hit)}`);
  }

  // Dialogs, drawers, and popovers must fit, and never hide a commit or irreversible action.
  for (const dialog of document.querySelectorAll('[role="dialog"], .filter-popover')) {
    if (!visible(dialog)) continue;
    const d = dialog.getBoundingClientRect();
    const fits = d.top >= -1 && d.left >= -1 && d.bottom <= window.innerHeight + 1 && d.right <= window.innerWidth + 1;
    const hidden = [];
    for (const action of dialog.querySelectorAll(".button.primary, .button.danger, button.danger")) {
      if (getComputedStyle(action).display === "none") continue;
      const a = action.getBoundingClientRect();
      let clipped = a.bottom > window.innerHeight + 1 || a.top < -1;
      for (let n = action.parentElement; n && n !== dialog.parentElement; n = n.parentElement) {
        const s = getComputedStyle(n);
        if (["hidden", "auto", "scroll", "clip"].includes(s.overflowY)) {
          const c = n.getBoundingClientRect();
          if (a.bottom > c.bottom + 1 || a.top < c.top - 1) clipped = true;
        }
      }
      if (clipped) hidden.push(text(action).slice(0, 40));
    }
    out.dialogs.push({ dialog: dialog.getAttribute("aria-labelledby") || name(dialog), fits, hiddenActions: hidden, height: Math.round(d.height) });
  }
  return out;
};
