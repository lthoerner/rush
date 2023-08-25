# Contributing to Rush
If you are looking to contribute to the Rush project, this document lays out some guidelines which
we hope will help you spend your time effectively/productively.

## How to Contribute
Contributions should be made by forking the project and making pull requests (PRs) from your fork.
If you are new to Git/GitHub and are not sure how to do this,
[this video](https://youtu.be/nT8KGYVurIU) should help.

Maintainers may make PRs directly from a branch on the repository itself, and *very* occasionally
make commits directly to the `main` branch, but this is highly discouraged.

Once a PR has been made, it requires the review of one maintainer to be merged. The review process
should take no more than two days for simple PRs; *do not hesitate* to bug me (@lthoerner) to review
your PR, especially if it is past this two-day threshold. If the PR is more complex or invasive to
the codebase, or if changes must be made prior to merging, the review process may take longer than
otherwise expected.

Please make sure that the *"Allow edits by maintainers"* box is checked when submitting your PR, as
it allows us to make quick alterations to your code without formally requesting changes.

## Development Environment

### System requirements
- Be running Linux, MacOS, or another compatible Unix-derivative OS for the shell to work. We do not currently have plans to work on Windows support, but if you are developing on Windows, you can use WSL.
- Have Cargo installed on your system.

### Editor setup
It is not required, but is highly recommended to use an editor/IDE which supports plugins,
particularly those listed as "essential." Examples are Visual Studio Code (VSCode), CLion, and
Neovim. The repository contains a folder called `.vscode/` which provides a recommended
configuration for VSCode.

#### Essential plugins
- *rust-analyzer*: Rust language support/language server
- *CodeLLDB*: Rust debugger
- *Error Lens*: Displays errors by highlighting lines instead of requiring a mouse-over on the offending code

#### Recommended plugins
- *Even Better TOML*: TOML language support, makes working on the manifests easier
- *Better Comments*: Highlights `// ?` comments in blue, `// *` comments in green, `// $` comments in red, and `// TODO:` comments in yellow
- *Hide files*: Allows you to hide files that clutter your workspace, such as `Cargo.lock`, `.gitignore`, `LICENSE.md`, etc.
- *GitHub Copilot*: Allows for quick and dirty descriptive comments, refactoring, method implementations, etc. by generating code for you (this is $10/mo but free if you are a student)
- *GitLens*: Adds inline annotations that tell you information about the Git history of the line you are editing

#### Honorable mentions
- *GitHub Theme*: A GitHub-based syntax highlighting theme to make the editor a little easier on the eyes
- *Discord Rich Presence*: Displays VSCode in Discord's "rich presence" (game activity) section on your profile

## Code Requirements
These are the "rules" for contributions in this project - exceptions can be made, but generally
speaking any and all contributions should adhere to these tenets. There will be a section for style
"recommendations" soon, which will encompass tips and helpful hints rather than actual rules.

1. Use Clippy as your linter. In VSCode, you can do this in **Preferences** > "**Rust Analyzer >
   Check: Command**" > "**clippy**". Clippy warnings should either be rectified or should be marked
   with `#[allow]` attributes in order to prevent busy yellow marks all over the workspace file
   tree. Only use `#[allow]` if a lint is truly unhelpful/its suggestion should not be implemented.
2. Do not use macros without consulting a maintainer beforehand. Macros make the codebase both
   difficult-to-maintain and relatively unreadable, especially for beginner Rust developers.
3. Make sure your code is commented to a level which allows any developer to mentally parse it, even
   if they do not have prior experience in Rust. On the other hand, do not leave excessive comments
   for self-descriptive code.
4. Ensure that your submitted PRs do not contain `.unwrap()`, `.expect()`, `panic!()`, or any other
   code which will allow a panic to occur. Errors should be at the very least passed up the
   callstack, and preferably they should be handled.
6. Comments must follow the format `// <message>`, and use standard English language conventions
   (capitalization, punctuation, etc.). You can also use `// $ <message>` to highlight a comment in
   red, `// * <message>` to highlight a comment in green, and `// ? <message>` to highlight a
   comment in blue. These are used for high-priority warnings, important clarifications to code, and
   "notes to self," respectively. Usually, only `// *` comments are suitable for merging.
7. `TODO`s must be commented in the appropate format (`// TODO: <message>`) and are generally not
   suitable for merging. Please submit complete work whenever remotely possible.
8. Do not use `unsafe`.

## Communication
The team communicates mostly via [Discord](https://discord.gg/KphQhFeKqv) for informal discussions,
and via GitHub for formal discussions. GitHub Discussions and Issues can be thought of forum
threads. Additionally, PR-specific communications should be held on their respective GitHub thread.
