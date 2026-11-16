---
name: framework-draft
description: "Explicit framework workflow: draft"
disable-model-invocation: true
---

> **Claude Code:** Same intent as Cursor `/draft`. When customizing, keep in sync with `.cursor/commands/draft.md`.

# /draft — Turn a work item into the right written artifact

Use when an item arrives without a decided shape: an action item, task, issue, TODO comment,
backlog line, meeting note, or a one-sentence request. This command decides **which** artifact
the item deserves — design doc, spec, plan, or none — then produces it and hands off.

`/spec`, `/plan`, and `/tasks` each assume you already know you need them. This is the gate before
them. It sits after `/decompose` (which splits epics) and before `/clarify`.

## Inputs

- The raw item text, verbatim (paste it back at the top of your output).
- Its source and ID: backlog ref from `.agent/backlog/items.md`, issue number, `file:line`
  for a TODO, or `ad hoc`.
- Repo conventions: `AGENTS.md`, `CLAUDE.md`, `.rules`, `docs/CANONICAL_SOURCES.md`.
- Any code the item points at. Read it before writing about it.

## Step 1 — Restate and triage

Restate the item as an outcome, not an activity: "<user or system> can <observable behavior>",
not "refactor the X module".

Then classify, with one sentence of justification:

| Class | Signal | Artifact to produce |
|-------|--------|---------------------|
| **Trivial** | Typo, obvious one-liner, dead link, single unambiguous edit | None. Say "no doc needed", state the change, stop. |
| **Small** | Clear behavior, one module, no design choices | **Plan** only |
| **Medium** | Behavior needs pinning down, or acceptance is arguable | **Spec** → **Plan** |
| **Design-bearing** | Two or more viable approaches, a new interface/schema/data flow, cross-crate impact, or a hard-to-reverse choice | **Design doc** → **Spec** → **Plan** |
| **Epic** | Multiple independent work streams, more than roughly a week | Run `/decompose` first, then `/draft` per subtask |

Over-documenting a Small item is a failure in the same way under-documenting a design-bearing one
is. On a genuinely borderline call, pick the lighter class and name what would push it heavier.

## Step 2 — Ask only blocking questions

At most **five**, only where different answers produce materially different work. For each: the
question, why it matters, the recommended default, and the consequence of that default. Everything
else becomes a written **Assumption** — proceed, do not block.

## Step 3 — Write the artifact(s)

### Design doc

Only for design-bearing items. Sections, in order:

1. **Context** — What exists today, citing real paths (`crates/foo/src/bar.rs:42`). No speculation.
2. **Problem** — What breaks, is slow, or is impossible now, with the evidence.
3. **Goals** — Bulleted, each observable.
4. **Non-goals** — What this deliberately does not do.
5. **Options considered** — At least two, plus "do nothing". For each: sketch, what it costs,
   what it buys, what it forecloses. Be honest about the one you will reject.
6. **Decision** — The chosen option and the deciding factor; a paragraph, not a restatement
   of the options.
7. **Design** — Interfaces, data shapes, state transitions, failure modes. Enough that a reviewer
   can attack it without reading the implementation.
8. **Migration / rollout** — Compatibility, flags, ordering, and how to back out.
9. **Risks** — Each with a trigger and a mitigation. "Unresolved" is valid if you name who resolves it.
10. **Open questions** — Marked `OPEN QUESTION`, each with an owner.

### Spec

Follow [`spec.md`](spec.md) exactly: summary, stakeholders, product scenario, non-goals, user
stories, `AC-n` acceptance criteria in EARS form, assumptions and dependencies, open questions.
Every `AC-n` is one `shall` sentence, testable without interpreting intent. Flag any criterion you
cannot make observable as **AMBIGUOUS** rather than smoothing it over.

### Plan

Follow [`plan.md`](plan.md): goal, files and areas to touch, ordered approach, constitution check
against `AGENTS.md` and `.agent/SAFETY.md`, failing test stubs (paths and assertion names) derived
from the `AC-n` IDs, `/tasks` handoff decision, rollback.

## Rules

- **Read before writing.** Every claim about current behavior cites a path or command output.
  If you did not look, write `unverified` next to it.
- **Do not invent contracts.** Check `docs/CANONICAL_SOURCES.md` before naming an API path,
  config key, schema name, or status value.
- **No solution in the spec, no requirements in the plan.** "We should use X" belongs in the
  design doc or the plan, never in a spec.
- **Size honestly.** A design doc for a two-line fix is waste; say so and downgrade the class.
- **Keep it traceable.** Design decision → `AC-n` → `T-n` → test, with nothing orphaned in
  either direction.
- **Preserve scope.** Anything you notice outside the item goes in **Out-of-scope follow-up**,
  not into the artifact.

## Output format

```
## Item
<verbatim item> — source: <ref>

## Restated outcome
<one sentence>

## Class: <Trivial | Small | Medium | Design-bearing | Epic>
<one-sentence justification>

## Blocking questions
<up to five, or "none">

## Assumptions
<explicit list, or "none">

---
<the artifact(s), in order: design doc → spec → plan>
---

## Out-of-scope follow-up
<items to file separately, or "none">

## Next command
<one of /clarify, /tasks, /implement, /decompose — and why>
```

## Done when

- The class is justified and the artifact set matches it — no ceremony, no gaps.
- Every acceptance criterion is one observable EARS sentence with a stable `AC-n` ID.
- Every design decision names the option it beat and why.
- Another developer could execute the plan without asking a question.
- Unknowns appear as `OPEN QUESTION` or **Assumption**, never as confident prose.

## Note

Design docs are review artifacts, not implementation records. Save to `docs/design/<slug>.md` only
if the user asks; otherwise leave the output in the conversation for review.
