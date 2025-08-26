# Tauri Contributing Guide

Hi! We, the maintainers, are really excited that you are interested in contributing to Tauri. Before submitting your contribution though, please make sure to take a moment and read through the [Code of Conduct](CODE_OF_CONDUCT.md), as well as the appropriate section for the contribution you intend to make:

- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Development Guide](#development-guide)

## Issue Reporting Guidelines

- The issue list of this repo is **exclusively** for bug reports and feature requests. Non-conforming issues will be closed immediately.

- If you have a question, you can get quick answers from the [Tauri Discord chat](https://discord.com/invite/tauri).

- Try to search for your issue, it may have already been answered or even fixed in the development branch (`v2`).

- Check if the issue is reproducible with the latest stable version of Tauri. If you are using a pre-release, please indicate the specific version you are using.

- It is **required** that you clearly describe the steps necessary to reproduce the issue you are running into. Although we would love to help our users as much as possible, diagnosing issues without clear reproduction steps is extremely time-consuming and simply not sustainable.

- Use only the minimum amount of code necessary to reproduce the unexpected behavior. A good bug report should isolate specific methods that exhibit unexpected behavior and precisely define how expectations were violated. What did you expect the method or methods to do, and how did the observed behavior differ? The more precisely you isolate the issue, the faster we can investigate.

- Issues with no clear repro steps will not be triaged. If an issue labeled "need repro" receives no further input from the issue author for more than 5 days, it will be closed.

- If your issue is resolved but still open, don't hesitate to close it. In case you found a solution by yourself, it could be helpful for others to explain how you fixed it.

- Most importantly, we beg your patience: the team must balance your request against many other responsibilities — fixing other bugs, answering other questions, new features, new documentation, etc. The issue list is not paid support and we cannot make guarantees about how fast your issue can be resolved.

## Pull Request Guidelines

- You have to [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

- It's OK to have multiple small commits as you work on the PR - we will let GitHub automatically squash it before merging.

- If adding new feature:
  - Provide convincing reason to add this feature. Ideally you should open a suggestion issue first and have it greenlighted before working on it.

- If fixing a bug:
  - If you are resolving a special issue, add `(fix: #xxxx[,#xxx])` (#xxxx is the issue id) in your PR title for a better release log, e.g. `fix: update entities encoding/decoding (fix #3899)`.
  - Provide detailed description of the bug in the PR, or link to an issue that does.

- If the PR is meant to be released, follow the instructions in `.changes/readme.md` to log your changes. ie. [readme.md](https://github.com/tauri-apps/plugins-workspace/blob/v2/.changes/readme.md)

## Development Guide

**NOTE: If you have any question don't hesitate to ask in our Discord server. We try to keep this guide to up guide, but if something doesn't work let us know.**

### General Setup

First, [join our Discord server](https://discord.com/invite/tauri) and let us know that you want to contribute. This way we can point you in the right direction and help ensure your contribution will be as helpful as possible.

To set up your machine for development, follow the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/) to get all the tools you need to develop Tauri apps. The only additional tool you may need is [PNPM](https://pnpm.io/).

Next, [fork](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/fork-a-repo) and clone [this repository](https://github.com/tauri-apps/plugins-workspace/tree/v2).

### Developing and testing

The easiest way to test your changes is to use the [example app](https://github.com/tauri-apps/plugins-workspace/tree/v2/examples/api) example app. It automatically rebuilds and uses your local copy of the plugins. To run it follow the instructions inside [README.md](https://github.com/tauri-apps/plugins-workspace/blob/v2/examples/api/README.md).

To test local changes against your own application simply point the plugin create to your local repository, for example:

`tauri-plugin-sample = { path = "path/to/local/tauri-plugin-sample/" }`
