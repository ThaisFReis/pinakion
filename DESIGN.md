# DESIGN

One look, used by the landing page, the ticket page and the video. It is derived from `THESIS.md`, so when the thesis changes, this changes next. It was last aligned on 2026-10-26 (T-043), after the thesis became the sealed draw (DEC-015, DEC-029).

---

## The institution we want to express

The page should read as a contemporary public record: legible, open to examination, and calm enough to read when a decision about a person matters. It speaks first to **an open-source project with a code of conduct that has to judge one of its own** (`THESIS.md`). Grant programs, protocols and marketplaces come second, in one *also serves* line. The regulated-institution audience is gone with the thesis row that carried it. Avoiding crypto clichés follows the human's request of 2026-10-25.

The interface should express four things.

- **The guarantee, in its own order.** First, nobody chooses who judges you. Second, nobody knows who until they have sat and judged. Third, anyone recomputes the panel afterwards. Each screen follows that order and introduces the machinery only after it.
- **Equal standing.** Roster entries carry the same visual weight. A seat is shown as a number (`y`), not as a person, and nobody is ranked by importance.
- **Sealed until the verdict.** No screen puts a roster name beside a seat before that juror revealed. A seat with no reveal reads *em silêncio* and shows only its `y` and seat account. The ticket page follows this because the ledger does.
- **Accountability.** The verdict of the check (*Confere* / *Não confere*), the evidence links and the exact limits of the check stay together, on the same screen.

The material reference stays cool limestone and oxidised bronze. The historical slot motif is a quiet editorial illustration. The second Athenian lot, the acorn that sends a juror to one court, is a fitting image, but it is used for *no choosing*, never as a claim of Athenian secrecy (*Ath. Pol.* 63–65). None of this implies state authority, judicial certification or institutional endorsement.

Avoid token imagery, neon, gradients, padlocks and shields, decorative "zero-knowledge" visuals, speculative price charts, wallet-first navigation, gavels and courthouse columns. All illustrations are original geometric SVG, and none represents a live roster. Do not invent endorsements, adoption figures or institutions as social proof.

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
- `--kept` and `--cast` carry meaning, so they **never** carry it alone. Pair each with a label and a shape, such as a filled disc against a hollow ring, so a reader who cannot tell them apart still reads the word.
- **The juror states** are always a word in a bordered tag, and the colour only reinforces the word:

  | State | Word on screen | Border and text |
  |---|---|---|
  | seated and revealed | *sentou · revelou* | `--patina` |
  | revealed, not seated | *revelou* | `--line`, `--ink` |
  | below `T`, never claimed | *sorteado e ausente* | `--bronze` |
  | no reveal | *em silêncio* | `--bronze` |
  | recused | *recusou* | `--line`, `--muted` |
  | impeded before lock | *impedido* | `--line`, `--muted` |

  `--bronze` marks the two states a reader should question, *absent* and *silent*. It is still a highlight, not a second accent.
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

The landing page has a deliberate reading order:

1. **Purpose.** The hero states the first sentence of the guarantee: *Ninguém escolhe quem te julga.*
2. **The three sentences.** The guarantee, one principle per sentence, each paired with the mechanism behind it.
3. **The limits, on the same screen.** The single-party setup, the seat-account funding and the missing relayer, the loss of `s`, off-chain deliberation, testnet only.
4. **The sealed run on testnet.** What happened on 2026-10-26, in counts, and a way into the ticket page.
5. **The open draw it began with.** The Fisher-Yates recomputation, labelled as `beacon-verifier`'s.
6. **Closing.**

The case page (`web/processo.html`, T-051) is the demo link. Its reading order:

1. **The verdict first.** *Procedente.* or *Improcedente.*, with the score in one line, then how many checks the browser just ran.
2. **The case in five figures.** Roster, seats and level, attendance and quorum, the score, the reveals.
3. **Nine phases, in the order they happened:**
   1. the roster;
   2. the lock;
   3. the complaint and the record;
   4. the sealed draw;
   5. the seats;
   6. the quorum;
   7. the vote;
   8. the verdict;
   9. the reveal.

   Each phase has the same three parts: what happened, said plainly; how it is checked, as ticked lines; and the transactions.
4. **The vote phase names Tessera** in a bronze-ruled box, before anything else in that phase. Tessera is the author's earlier module, used unchanged. Bronze is the colour this page uses for what a reader should weigh.
5. **The limits, then the record's path and the command that checks it.**

A sticky step rail mirrors the nine phases with a tick each, and marks the one in view. The documents open in a dialog with the hash they were matched against. **Present** shows one phase per screen, with arrow and space keys and an auto-advance, so a video can be recorded from the page itself.

The ticket page (`web/bilhetes.html`) is the record itself. It opens with the verdict of the check, then the facts (root, N and k, round, seed, `T`, panel), then the table of people, then the table of claims, then its limits.

- An editorial shell with a maximum width of 1280px and 28px side gutters, 20px on small screens. These are design choices, not product measurements.
- A two-column introduction pairs the purpose with a restrained roster illustration. At 680px and below the content stacks.
- Principles sit on rules, not inside marketing cards. On small screens each one becomes a numbered row.
- Tables of people and claims scroll inside their own box on small screens, never the page. Addresses and hashes are shortened in the middle (`GAI2OV…F3SM`) and always link to the explorer.
- Status colours always come with shape and text. A *Confere* says the recomputation matches the ledger's events. It does not certify a fair process or an honest setup.
- Native semantic elements, a skip link, visible focus, labelled inputs and a live status region support keyboard and assistive-technology use.
- Light and dark share the same hierarchy. The page follows the system preference first, and a labelled theme button can override it for the session.
- `web/style.css` is the implemented token source. A page adds only the rules it needs, inline, and literal colours belong only in theme token declarations.

---

## Voice

The same rules as `CLAUDE.md`, applied to what the reader sees.

- Say what it does. "Recompute the panel", not "leverage verifiable randomness".
- Never claim secure, audited, trustless or production-ready. §2 of the spec lists what may be claimed.
- **Indistinguishable within the panel**, never *anonymous*. Say **recomputable among those who revealed**, never *fully recomputable*. In Portuguese: *indistinguível dentro do painel*; *recalculável entre quem revelou*.
- Limits are part of the page, not a footnote: what was not tested, what is still assumed.
- Short sentences. No em-dash asides, no "not X but Y", no scare quotes.
- Portuguese for the Brazilian-facing documents, English for the repository and the submission. Legal citations keep their original wording.

---

## Implementation and limits

`web/index.html` and `web/style.css` implement this direction (T-017, revised in T-043). `web/processo.html` and `web/processo.mjs` are the case (T-051, T-050). `web/bilhetes.html` is the sealed record (T-042). `web/sorteio.mjs`, `web/poseidon.mjs` and `web/arvore.mjs` hold the recomputations.

None of this is a production deployment. The pages manage no roster and run no case: they read a committed record of one. Nothing on them signs or sends a transaction. Neither page verifies the drand BLS signature in the browser: the contract does that at `seal`. The ticket page reads a committed record of the testnet run, not a live RPC. The open-draw example keeps its original contract and transaction links.

Typography loads from Google Fonts, with local fallback fonts, and the pages stay readable if the font service is unavailable.
