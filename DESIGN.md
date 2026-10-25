# DESIGN

One look, used by the contract's page, the landing page and the video. Derived from `THESIS.md`; when the thesis changes, this changes next.

---

## The institution we want to express

A contemporary public record: legible, open to examination, and calm enough to read when a decision matters. The audience includes communities, DAOs, grant programs, funds and regulated institutions. This shared direction follows the human's request on 2026-10-25 to avoid crypto clichés and design around what a governance needs to communicate.

The interface should express:

- **Legitimacy through procedure.** Explain how the deciding panel is formed before presenting technical machinery.
- **Equal standing.** Give roster entries the same visual weight; distinguish a selected position without ranking people by importance.
- **Accountability.** Keep the result, the evidence link and the exact limits of the check together.
- **Continuity.** Use a consistent visual language across the introduction, the form and the record.

The material reference remains cool limestone and oxidised bronze. The historical slot motif becomes a quiet editorial illustration. It does not imply state authority, judicial certification or institutional endorsement.

Avoid token imagery, neon, gradients, speculative price charts, wallet-first navigation, decorative security seals, gavels and courthouse columns. All illustrations are original geometric SVG; none represents a live roster.

The user asked for all audiences rather than a single sector. Use one shared identity. Do not invent endorsements, adoption figures or institutions as social proof.

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
  --line:    #C9CDC2;   /* rules, borders */
  --patina:  #2E6B5C;   /* primary accent — oxidised bronze */
  --bronze:  #8A6A3B;   /* secondary — fresh metal, used sparingly */
  --kept:    #FFFFFF;   /* white ball: seated, kept, verified */
  --cast:    #1A1B17;   /* black ball: dismissed, excluded, rejected */
  --accent-surface: #E0E8E0;
  --on-accent: #FFFFFF;
  --error: #8B3E31;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    /* dark — the slab in shadow */
    --stone:  #16181A;
    --slab:   #1E2126;
    --ink:    #E9EAE4;
    --muted:  #A5AA9F;
    --line:   #41463E;
    --patina: #8DC9B5;
    --bronze: #CBA468;
    --kept:   #F2F3EE;
    --cast:   #0C0D0F;
    --accent-surface: #273B34;
    --on-accent: #13231E;
    --error: #F1A594;
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
| Body and UI | **Public Sans**, 400 / 500 / 600 / 700 | A typeface drawn for public institutions. The project is one. |
| Data | **IBM Plex Mono**, 400 / 500 | Panels, hashes, round numbers. `font-variant-numeric: tabular-nums` wherever digits line up. |

```html
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@600&family=Public+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&display=swap">
```

```css
--display: "Cormorant Garamond", Georgia, "Times New Roman", serif;
--body:    "Public Sans", system-ui, -apple-system, "Segoe UI", sans-serif;
--mono:    "IBM Plex Mono", ui-monospace, "SF Mono", Menlo, monospace;
```

**Scale**

| Token | Size | Use |
|---|---|---|
| `--t-hero` | `clamp(3.25rem, 6.8vw, 6.25rem)` | the purpose statement, once per page |
| `--t-display` | `clamp(2.2rem, 4vw, 3.5rem)` | section openers |
| `--t-h2` | `1.6rem` | headings inside a section |
| `--t-body` | `1rem` / 1.65 | running text, max 65 characters wide |
| `--t-small` | `0.875rem` | notes, captions |
| `--t-label` | `0.6875rem`, `letter-spacing: 0.15em`, uppercase | eyebrows, field labels |
| `--t-data` | `0.8125rem` mono | panels, hashes, measurements |

Headings take `text-wrap: balance`. Cormorant is a display face: never below `--t-h2`.

---

## Logo

The kleroterion's slot column, with one slot drawn. It is the machine and the act at the same time, and it survives at 16px.

```svg
<svg viewBox="0 0 24 28" fill="none" aria-hidden="true">
  <rect x="2.5" y="2.5" width="19" height="23" rx="1" stroke="currentColor"/>
  <path d="M7 8h10M7 20h10" stroke="currentColor" stroke-width="1.5"/>
  <path d="M7 14h10" stroke="currentColor" stroke-width="3"/>
</svg>
```

- Draws in `currentColor`, so it inherits `--ink` in text and `--patina` when the mark stands alone.
- Clear space on every side: the height of one slot.
- The filled slot is always the **second** of three. Not centred by accident — drawn.
- Wordmark: *Pinakion* in Cormorant Garamond 600, `letter-spacing: -0.035em`. The Greek **πινάκιον** may appear once, as a subordinate line in `--muted`, never as the mark itself.

---

## Layout

The page has a deliberate reading order: purpose → proposed procedure → working example → evidence and limitations. The testnet status appears inside the example. Proposed functionality is explicitly marked as under development.

- An editorial shell with a maximum width of 1280px and 28px side gutters; 20px gutters on small screens. These are design choices, not product measurements.
- A two-column introduction pairs the purpose with a restrained roster illustration. At 680px and below the content stacks; the figure becomes a compact horizontal composition.
- Three procedural statements sit on rules rather than enclosed marketing cards. On small screens each becomes a numbered row.
- The verification record places inputs beside the recomputed panel, then stacks them on small screens. The selected positions use equally sized cells and tabular numerals.
- Signature, seed, comparison input and technical recipe are accessible through native disclosure controls. Evidence links and the limits of the check remain visible.
- Shape and text accompany status colours. The success state says that panels match the supplied data; it does not certify an honest or fair process.
- Native semantic elements, a skip link, visible focus, labelled inputs and a live status region support keyboard and assistive technology use.
- Light and dark themes share the same hierarchy. Follow the system preference initially; a labelled theme button can override it for this page session.
- Use `web/style.css` as the implemented token source. Literal colours belong only in theme token declarations.

---

## Voice

The same rules as `CLAUDE.md`, applied to what the reader sees.

- Say what it does. "Recompute the panel", not "leverage verifiable randomness".
- Never claim secure, audited, trustless or production-ready. §2 of the spec lists what may be claimed.
- Limits are part of the page, not a footnote: what was not tested, what is still assumed.
- Short sentences. No em-dash asides, no "not X but Y", no scare quotes.
- Portuguese for the Brazilian-facing documents, English for the repository and the submission. Legal citations keep their original wording.

---

## Implementation and limits

T-017 applies this direction to `web/index.html` and `web/style.css`. The existing independent recomputation remains in `web/sorteio.mjs`.

This is a local design revision for review. It is not a production deployment, and the page does not add roster management, case handling, cryptographic signature verification in the browser or a live read from Stellar. The example retains its documented original contract and transaction links; the method disclosure distinguishes that example from the later pinned-key contract.

Typography loads from Google Fonts as it did before this revision, with local fallback fonts. The page remains readable if the font service is unavailable.
