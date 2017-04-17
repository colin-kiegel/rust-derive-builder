# DEV Tools

It's very useful to set `dev/githook.sh` as a `pre-push` hook.

On Linux, do this:

```bash
(cd .git/hooks && ln -s ../../dev/githook.sh pre-push)
```

As macOS doesn't support symlinks in `readlink`, do this:

```bash
(cd .git/hooks && echo $'#!/bin/bash
 dev/githook.sh' > pre-push)
```

This will basically do all the tests that travis would do, before the push is
executed.

Running the tests might take a little while, because the script will update
each required toolchain via rustup if neccessary. It also buffers all writes to
stdout/stderr and only prints it on failure. A successful run should look like:

```
$ git push
II: Working directory is... ✓
II: Running tests on stable... ✓
II: Running tests on beta... ✓
II: Running tests on nightly... ✓
II: Running dev/compiletests.sh... ✓
II: Running dev/checkfeatures.sh... ✓
OK: All checks passed!
```

## Tips:

* You can tell git to skip all tests via the `--no-verify` argument,
  e.g. `git push --no-verify`.
* You can also run `dev/githook.sh` manually at any time, without
  registering it as a git hook. But you still have to do the configuration.

## Configuration

The `dev/githook.sh` requires the following git config variables to be set,
e.g.:

```bash
git config hooks.usecolor true
git config hooks.rustup true
git config hooks.checkformat false # We don't follow _all_ of these suggestions
git config hooks.checkstable true
git config hooks.checkbeta true
git config hooks.checknightly true
git config hooks.compiletests true
git config hooks.checkfeatures true
```
