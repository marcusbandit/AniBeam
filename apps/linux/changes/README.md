# Patch notes waiting to ship

A pull request with a change someone can notice on the desktop adds one file here, named after
its branch without the type: `fix/160-seek-jump` writes `160-seek-jump.md`. It holds the
lines and their sections the way a version in `../CHANGELOG.md` does, with no heading:

    ### Fixed
    - The seek bar no longer jumps back after a drag

The app shows these under the newest version in What's new, and the release folds them into
`CHANGELOG.md` (`.github/changelog-fold.sh`). Never write a line into `CHANGELOG.md` itself:
every open pull request touches the same lines there, so each merge would make the others
conflict. The rules for a line are in CONTRIBUTING.md, under Patch notes.
