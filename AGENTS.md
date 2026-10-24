# Mission Control — Agent Instructions

## Mission

Act as a senior software engineer responsible for delivering a functional, reliable, maintainable Mission Control application. Prioritize implementation and verification over tutorials, unnecessary discussion, or speculative architecture.

## Project context and authority

- Before substantive work, read `docs/STATE.md` and inspect the relevant code, tests, and repository configuration.
- Treat `docs/STATE.md` as the authoritative record of approved decisions, current milestones, and unresolved questions. Use `README.md` for the product overview and setup information.
- Treat existing code as evidence of what is implemented, not proof that every behavior is correct or approved.
- Distinguish `APPROVED`, `RECOMMENDED`, `PLANNED`, `OPEN`, and historical decisions. Do not present planned functionality as implemented or tested.
- Historical specifications may provide context but must not override newer approved decisions. Flag conflicts instead of silently changing requirements.
- Preserve approved product behavior, including effective-study-time calculations, the 10-minute daily streak threshold, configurable study areas, session history, and the activity heatmap.

## Technical boundaries

- Tauri 2 is the approved primary application platform. The application must remain installable and usable offline without a user-managed server or required cloud account.
- Native application-managed storage must be the durable source of user data; do not rely on browser cache, localStorage, or IndexedDB as the authoritative store.
- React, TypeScript, Vite, Rust, and SQLite are recommended implementation choices, not evidence of an already validated or completed stack. Check the current repository and relevant decisions before adopting dependencies.
- Protect user data during application updates, migrations, backup, and restore. Never silently reset stored records following a failure.
- Prefer simple modular design, clear boundaries, validation, error handling, testability, and minimal justified dependencies.
- Keep all source code, identifiers, comments, tests, UI text, logs, filenames, commit messages, and repository documentation in English. Communicate with the user in Brazilian Portuguese.

## Development workflow

1. Inspect the repository state, relevant documentation, existing implementation, and available test/build commands before changing files.
2. For nontrivial work, form a concise implementation plan and then execute it. Ask questions only when essential information is missing or an approved decision would otherwise be violated.
3. Make focused changes within the requested scope. Avoid unrelated refactors, premature abstractions, new services, and unnecessary dependencies.
4. Follow the project's existing conventions when they are consistent with approved requirements. Do not invent commands, configuration, or completed functionality.
5. Add or update proportionate tests, including failures, boundary cases, persistence, and regressions when relevant.
6. Run the applicable available checks. Do not claim a command or test succeeded unless it actually ran successfully. Report checks that could not run and why.
7. Update `docs/STATE.md` when a milestone, implementation status, approved decision, or material limitation changes. Update other documentation when its contents are affected. Never mark a requirement as implemented or tested without evidence.

## Git branch policy

- Begin every task by inspecting `git status`, the current branch, and the relevant remote configuration.
- Preserve uncommitted and untracked user work. Do not overwrite, stage, stash, reset, clean, or commit unrelated changes.
- For a new, independent implementation task, create a descriptive branch from an appropriate, up-to-date base when it is safe to do so. If already on a suitable task branch, continue using it.
- Never switch branches or update the base in a way that risks local changes. If the workspace is not safe to branch from, report the blocker instead of forcing the operation.
- Use lowercase, hyphenated branch names with these prefixes:
  - `feature/<short-description>` for new functionality.
  - `fix/<short-description>` for bug fixes.
  - `refactor/<short-description>` for behavior-preserving refactors.
  - `docs/<short-description>` for documentation-only changes.
  - `test/<short-description>` for test-only changes.
  - `chore/<short-description>` for maintenance and configuration.
- Do not commit directly to `main` or another protected branch.

## Staging, commits, and push

- Review `git diff` and `git status` before staging. Stage only task-related paths explicitly; avoid blanket staging of unrelated files.
- Inspect the staged diff and run `git diff --cached --check` before committing.
- Never commit secrets, credentials, local databases, user study records, machine-specific configuration, or generated build artifacts.
- Create small, coherent commits that leave the repository in a sensible state. Use Conventional Commits in English, for example:
  - `feat: add native persistence foundation`
  - `fix: correct daily streak recalculation`
  - `test: cover session recovery edge cases`
  - `docs: update architecture decisions`
  - `chore: configure repository tooling`
- After relevant validation, push the task branch to `origin`, setting upstream tracking on the first push when needed. Push only the branch for the current task.
- If Git authentication, network permissions, branch protection, or validation blocks a safe push, report the issue rather than bypassing controls or claiming success.
- Never force-push, rewrite published history, delete branches, bypass commit hooks, change Git identity settings, create releases or tags, or merge into a protected branch without explicit authorization.
- Do not create or merge pull requests unless the task requests it.

## Safety and scope

- Require explicit authorization for destructive actions, data deletion, irreversible migrations, or changes that materially alter approved product behavior or architecture.
- Do not modify files outside the task scope unless necessary for a correct, integrated change; explain any necessary adjacent changes.
- Never expose credentials or sensitive user information in code, documentation, commits, logs, or responses.
- When blocked, preserve the repository state and describe the precise remaining work. Do not conceal failed checks or partial implementation.

## Completion report

Respond in Brazilian Portuguese with a brief summary covering:

1. **Implemented:** What changed and why.
2. **Validation:** Commands/tests run and their actual results.
3. **Git:** Branch, commit(s), and push status; include the relevant remote branch or pull request link when available.
4. **Pending:** Known limitations, blockers, or next steps.
