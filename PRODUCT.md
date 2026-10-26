# Product

Strategic context for design work. Derived from `THESIS.md`, `DESIGN.md` and `docs/SPEC.md` §1–§2 on 2026-10-26 (T-056); nothing here is new, and when those change, this changes next. `DESIGN.md` holds the visual system.

## Register

product

The landing page (`web/index.html`) is the one brand surface. The case page, the ticket page and the live page are product surfaces: a reader is checking a record or running a case, and the design serves that task.

## Users

- **First:** an open-source project with a code of conduct that has to judge one of its own. The maintainer, the moderation team, the person judged, and the jurors drawn from the community. They arrive worried about fairness, often mid-conflict, on a laptop, reading in Portuguese or English.
- **Second:** any governance that decides about a person rather than a parameter: a grant program revoking a grant, a protocol judging a slashed validator, a marketplace delisting a seller who disputes it.
- **During the event:** hackathon judges with a few minutes per submission, who score demos rather than ideas. They open the case page link, want to see the case work and be checked, and want the limits stated plainly.

The job on any screen: read what happened, see that it was checked, and reach the evidence (the transaction, the document, the command) without trusting the page.

## Product Purpose

Pinakion is sealed jury selection for governance on Stellar. The guarantee, in its own order: nobody chooses who judges you; nobody knows who until they have sat and judged; afterwards, anyone recomputes the panel from the ledger, among those who revealed.

The pages do not manage a roster or run a case for anyone; they read a committed record of one case (the case page, the ticket page) or run one fictional case from the visitor's tab on testnet (the live page). Success is a stranger reaching *Confere* on their own machine, and understanding what that does and does not certify.

## Brand Personality

**Civic, legible, calm.** A contemporary public record: open to examination, calm enough to read when a decision about a person matters. Say what it does ("recompute the panel"), never what it promises. Limits are part of the page, never a footnote.

Vocabulary that holds: *indistinguishable within the panel*, never *anonymous*; *recomputable among those who revealed*, never *fully recomputable*. In Portuguese: *indistinguível dentro do painel*; *recalculável entre quem revelou*. Never *secure*, *audited*, *trustless* or *production-ready*.

## Anti-references

- Crypto product pages: token imagery, neon, gradients, padlocks and shields, decorative "zero-knowledge" visuals, price charts, wallet-first navigation.
- Courthouse kitsch: gavels, columns, scales of justice. Athens is cited for *no choosing*, never for authority.
- SaaS dashboards: hero metrics with gradient accents, identical card grids, modals as the first thought.
- Invented social proof: endorsements, adoption figures, institutions.

## Design Principles

1. **The guarantee sets the order.** Every screen introduces the three sentences before the machinery, in that order.
2. **Equal standing.** Roster entries carry the same weight; a seat is a number, never a person; nobody is ranked.
3. **Sealed until the verdict.** No screen puts a name beside a seat before that juror revealed.
4. **The check and its limits stay together.** *Confere* / *Não confere*, the evidence links and what the check does not certify sit on the same screen.
5. **Show the mechanism, not the pitch.** A judge scores what runs: the browser recomputing, the transaction landing, the document matching its hash.

## Accessibility & Inclusion

- Status colours never carry meaning alone: each pairs with a word in a bordered tag and, where two states contrast, a shape (filled disc against hollow ring).
- Native semantic elements, a skip link, visible focus, labelled inputs, a live status region; the presentation mode works from the keyboard.
- Light and dark share one hierarchy; the page follows the system preference and a labelled button overrides it for the session.
- `prefers-reduced-motion` turns off the only animations (the phase entrance, the live spinner).
- Fonts load from Google Fonts with local fallbacks; the pages stay readable without them.
- No WCAG level is claimed; none has been audited.
