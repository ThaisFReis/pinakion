# DESIGN

One look, used by the contract's page, the landing page and the video. Derived from `THESIS.md`; when the thesis changes, this changes next.

---

## The world

A small bronze plaque, engraved with a name, slotted into pale limestone. Left in a courtyard where anyone walking past could read it. Surviving pinakia are **green**, not gold — bronze oxidises, and the patina is what two and a half thousand years look like.

So the palette is **cool stone and oxidised bronze**, with fresh metal used sparingly. Not warm parchment, not gold.

And the randomiser is already a colour system: **black and white balls** fell through the tube, white keeping a row in service and black dismissing it. Those are the project's semantic colours, and they mean exactly what they meant in 370 BC.

---

## Colour

Every value is a token. Nothing uses a literal colour.

```css
:root {
  /* light — limestone in daylight */
  --stone:   #ECEDE8;   /* page ground */
  --slab:    #F7F8F5;   /* cards, panels, inputs */
  --ink:     #1A1B17;   /* text */
  --muted:   #5A5E55;   /* secondary text, labels */
  --line:    #D3D6CC;   /* rules, borders */
  --patina:  #2E6B5C;   /* primary accent — oxidised bronze */
  --bronze:  #8A6A3B;   /* secondary — fresh metal, used sparingly */
  --kept:    #FFFFFF;   /* white ball: seated, kept, verified */
  --cast:    #1A1B17;   /* black ball: dismissed, excluded, rejected */
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    /* dark — the slab in shadow */
    --stone:  #16181A;
    --slab:   #1E2126;
    --ink:    #E9EAE4;
    --muted:  #969A90;
    --line:   #2F3338;
    --patina: #63BBA4;
    --bronze: #CBA468;
    --kept:   #F2F3EE;
    --cast:   #0C0D0F;
    color-scheme: dark;
  }
}

:root[data-theme="dark"] { /* same dark values, so the toggle wins both ways */ }
```

**Rules**

- `--patina` is the accent. `--bronze` is a highlight, never a second accent — one bold place, everything else quiet.
- `--kept` and `--cast` carry meaning, so they **never** carry it alone: pair with a label and a shape (filled disc versus hollow ring). A reader who cannot distinguish them still reads the word.
- `--kept` on light needs a `--line` outline or it disappears into `--slab`.
- No gradients. The materials are stone and metal; neither graduates.

---

## Type

| Role | Face | Why |
|---|---|---|
| Display | **Cormorant Garamond**, 600 | High contrast, sharp serifs. Reads as cut into a surface rather than printed on one. |
| Body and UI | **Public Sans**, 400 / 500 / 700 | A typeface drawn for public institutions. The project is one. |
| Data | **IBM Plex Mono**, 400 / 500 | Panels, hashes, round numbers. `font-variant-numeric: tabular-nums` wherever digits line up. |

```html
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@600&family=Public+Sans:wght@400;500;700&family=IBM+Plex+Mono:wght@400;500&display=swap">
```

```css
--display: "Cormorant Garamond", Georgia, "Times New Roman", serif;
--body:    "Public Sans", system-ui, -apple-system, "Segoe UI", sans-serif;
--mono:    "IBM Plex Mono", ui-monospace, "SF Mono", Menlo, monospace;
```

**Scale**

| Token | Size | Use |
|---|---|---|
| `--t-hero` | `clamp(2.2rem, 6vw, 3.4rem)` | the one-liner, once per page |
| `--t-display` | `clamp(1.5rem, 4vw, 2.1rem)` | section openers |
| `--t-h2` | `1.3rem` | headings inside a section |
| `--t-body` | `1rem` / 1.6 | running text, max 65 characters wide |
| `--t-small` | `0.875rem` | notes, captions |
| `--t-label` | `0.72rem`, `letter-spacing: 0.11em`, uppercase | eyebrows, field labels |
| `--t-data` | `0.85rem` mono | panels, hashes, measurements |

Headings take `text-wrap: balance`. Cormorant is a display face: never below `--t-h2`.

---

## Logo

The kleroterion's slot column, with one slot drawn. It is the machine and the act at the same time, and it survives at 16px.

```svg
<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Pinakion">
  <rect x="3.5" y="2.5" width="17" height="19" rx="2"
        stroke="currentColor" stroke-width="1.5"/>
  <rect x="7" y="6" width="10" height="2.4" rx="1.2"
        stroke="currentColor" stroke-width="1.2"/>
  <rect x="7" y="10.8" width="10" height="2.4" rx="1.2"
        fill="currentColor"/>
  <rect x="7" y="15.6" width="10" height="2.4" rx="1.2"
        stroke="currentColor" stroke-width="1.2"/>
</svg>
```

- Draws in `currentColor`, so it inherits `--ink` in text and `--patina` when the mark stands alone.
- Clear space on every side: the height of one slot.
- The filled slot is always the **second** of three. Not centred by accident — drawn.
- Wordmark: *Pinakion* in Cormorant Garamond 600, `letter-spacing: 0.01em`. The Greek **πινάκιον** may appear once, as a subordinate line in `--muted`, never as the mark itself.

---

## Layout

- One column, `max-width: 68ch`, centred. The project is read, not operated.
- Side gutter `max(16px, env(safe-area-inset-left, 0px))` and the same on the right, set once on the outer wrapper, with vertical padding via `padding-block`.
- Spacing with flex or grid and `gap`, never per-element margins.
- Cards get `--slab`, a `--line` border, `border-radius: 6px`. Used by role — the panel, the verdict, the form — never on every block.
- Drawn positions render as a wrapping row of monospace chips, tabular. When a panel has been checked against an on-chain record, the chips take `--patina`.

---

## Voice

The same rules as `CLAUDE.md`, applied to what the reader sees.

- Say what it does. "Recompute the panel", not "leverage verifiable randomness".
- Never claim secure, audited, trustless or production-ready. §2 of the spec lists what may be claimed.
- Limits are part of the page, not a footnote: what was not tested, what is still assumed.
- Short sentences. No em-dash asides, no "not X but Y", no scare quotes.
- Portuguese for the Brazilian-facing documents, English for the repository and the submission. Legal citations keep their original wording.

---

## Where this is not applied yet

`web/index.html` was built before this file existed. It uses Newsreader, Public Sans and IBM Plex Mono on a blue accent, with its own grey ramp. It works, and it diverges from everything above.

Bringing it onto these tokens is a task, not a rewrite: the fonts link, the `:root` block and the chip colours. Until that is done, this file is the intent and that page is the exception — which is exactly the kind of disagreement `CLAUDE.md` says to record rather than leave quiet.
