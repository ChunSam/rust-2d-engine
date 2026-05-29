# Detailed agent operating rules

This document supplements the quick checklist in `AGENTS.md` with detailed operating
rules. Keep `AGENTS.md` under 200 lines; when detailed rules grow long, split them into
this document or a separate `docs/*.md`.

## 1. Default workflow

1. Explore: locate symbols and files with `rg` first, and read only the files you need.
2. Scope: classify the task as single-file, multi-file, public API, docs/release, or risky.
3. Plan: for multiple subsystems, public API, large changes, or unclear requirements, write a short plan before implementing.
4. Implement: prefer existing module boundaries and architecture patterns.
5. Verify: run the tests and doc checks that fit the change scope.
6. Report: leave a short summary of the change, key files, verification run, and any remaining risk or skipped verification.

## 2. Criteria by scope

| Scope | Criteria |
| --- | --- |
| Single-file edit | Handle directly in the main session when requirements are clear. |
| Multi-file feature | Narrow down and read related files first; split exploration/implementation/review into subagents if needed. |
| Public API change | Check `src/lib.rs` re-exports, examples, and doc impact. |
| Docs/release impact | Update whichever of `REFERENCE.html`, `README.md`, `docs/CHANGELOG.md`, `docs/HANDOFF.md` is needed. |
| Risky work | Do not proceed without prior confirmation. |

Risky work includes public API removal/rename, dependency/version changes, large refactors, file deletion, and destructive Git operations.

## 3. Using subagents

Use subagents freely. They are recommended especially for:

- work that requires exploring 3+ files
- features that touch multiple subsystems at once
- moving from a long conversation into actual code writing
- cases where splitting exploration, implementation, and review in parallel improves quality

The main agent does not accept subagent results as final. Final integration, conflict checking, test selection, and the completion report are the main agent's job.

A subagent prompt must always include:

1. Paths to edit or investigate
2. Architecture patterns to apply: borrow workaround, render layer separation, system registration order, etc.
3. The expected result and what behavior should change
4. The do-not-change scope, or the scope to only inspect

## 4. Verification criteria

Choose verification to match the change scope.

| Change type | Recommended verification |
| --- | --- |
| Docs only | Check links, doc structure, and that `AGENTS.md` is under 200 lines |
| Single-module logic | Related unit tests or `cargo test <module_or_test>` |
| Public API/example impact | Related tests + confirm examples still compile |
| Rendering/platform impact | `cargo build` where possible, and a WASM build check if needed |
| Release/packaging impact | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, `cargo doc --no-deps`, review a package dry run |

You do not always have to run the full gate for every task. But if you skip any verification, state it in the completion report.

## 5. Doc-update criteria

When public API, usage, examples, or release notes are affected, update docs alongside.

- Public API descriptions or example changes: `REFERENCE.html`
- User getting-started steps, requirements, or check commands: `README.md`
- Changes visible to release users: `docs/CHANGELOG.md`
- Dev history, architecture decisions, handoff notes: `docs/HANDOFF.md`
- Agent operating rules: `AGENTS.md` summary + details in this document

Content that could push `AGENTS.md` over 200 lines goes into a new `docs/*.md`, with only a one-line summary and link added to `AGENTS.md`.

## 6. Related-project checks

The default verification scope is this engine repo. By default, do not build or modify `rust-survivors`.

Check `rust-survivors` only when:

- the user explicitly requests it
- there is a clear possibility of a breaking change to the engine's public API
- the purpose of the change is to fix a `rust-survivors` integration issue

If you ran a check, note which commands or searches you used in the completion report.

## 7. Git rules

Run stage, commit, and push only when the user explicitly requests it. Do not revert changes made by someone else during the work.

Forbidden or requiring prior confirmation:

- destructive Git operations such as `git reset --hard`, `git checkout --`, force push
- reverting unrelated files
- auto-commits the user did not request
- large reformatting with unclear intent

## 8. Completion report format

Keep completion reports short and concrete.

- Change summary
- Key files
- Verification run
- Skipped verification or remaining risk

For docs-only changes you may skip code tests; instead report line counts, links, and doc-structure verification results.
