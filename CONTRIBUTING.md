# Contributing to `derive_builder`

Thank you for your interest in contributing to this crate!

## Pull Requests

Please make [pull requests] against the `master` branch.
[pull requests]: https://help.github.com/articles/using-pull-requests/

All pull requests should clearly describe what they intend to do, or link to
a github issue, where this is explained.

You should try to make sure your pull request passes all tests. Since some
tests are behind feature gates it's best to run the script `dev/githook.sh` as
described in `dev/README.md`. This script is intended to be primarily used as a
pre-push git hook, but it can be called manually as well.

Please follow this checklist
- update the `CHANGELOG.md` - add a section `[Unreleased]` at the top, if
  that's missing.
- update the documentation in `lib.rs` (optional: `README.md`)
- add unit tests to `derive_builder_core`, if appropriate
- add integration tests to `derive_builder` with different variants and also
  try to test possible side effects with other features of this crate.

### Early Feedback and Help

If you're stuck somewhere and need help or if you just want some early feedback,
you can either open an issue, or send a preliminary PR. In that case, please
mark it as **work in progress (WIP)** and state your questions.
