# Releasing `ezcli`

This document describes the recommended release workflow for `ezcli` so each release stays repeatable, easy to review, and easy to trace later.

## Versioning Conventions

- Use semantic-style version numbers such as `0.1.3`
- Use Git tags with a leading `v`, such as `v0.1.3`
- Keep the GitHub Release title the same as the tag, such as `v0.1.3`

## Pre-release Checks

Before releasing, confirm the following:

- The current changes are complete and the release scope is clear
- `CHANGELOG.md` includes the changes for this release
- If needed, `CHANGELOG.zh.md` has also been updated
- The `version` field in `Cargo.toml` has been updated to the target version
- The working tree is clean and does not contain temporary changes you do not want to ship

Useful command:

```powershell
git status --short
```

## Update Version and Changelog

1. Update the version number in `Cargo.toml`
2. Move the relevant `Unreleased` content in `CHANGELOG.md` into a versioned release section
3. Fill in the release date
4. If you maintain the Chinese changelog, keep `CHANGELOG.zh.md` in sync

For each release, focus the changelog on:

- Added: new capabilities
- Changed: behavior changes or implementation improvements
- Fixed: user-visible bug fixes

Do not turn the changelog into a copy of every small development commit.

## Commit the Release Changes

After confirming the release-related files, create one clear release commit.

Useful commands:

```powershell
git add Cargo.toml CHANGELOG.md CHANGELOG.zh.md
git commit -m "0.1.3"
```

If the release also includes other files that should ship together, add them in the same commit.

## Create the Git Tag

After the release commit is ready, create an annotated tag for that commit:

```powershell
git tag -a v0.1.3 -m "Release v0.1.3"
```

Check the tag:

```powershell
git show v0.1.3 --no-patch
```

## Push the Commit and Tag

Push the branch first, then push the tag:

```powershell
git push origin master
git push origin v0.1.3
```

If the default branch is renamed to `main` later, replace `master` with `main`.

## Create the GitHub Release

In the GitHub repository Releases page:

1. Click `Draft a new release`
2. Choose the existing tag, for example `v0.1.3`
3. Set the Release title to `v0.1.3`
4. Copy the matching version content from `CHANGELOG.md` into the Release notes
5. Do not repeat the line `## [0.1.3] - YYYY-MM-DD` in the Release body
6. If this is the newest version, check `Set as the latest release`
7. Click `Publish release`

Recommendations:

- Prefer the hand-written changelog as the Release body
- Do not rely entirely on GitHub auto-generated release notes

## Post-release Verification

After publishing, confirm at least the following:

- The version in `Cargo.toml` is correct
- The matching tag exists on GitHub
- The matching release appears in the GitHub Releases page
- The Release title, tag, and changelog content all match
- If needed, verify installation behavior such as `cargo install` or source downloads

## Recommended `ezcli` Release Commands

Example for releasing `0.1.3`:

```powershell
git status --short
git add Cargo.toml CHANGELOG.md CHANGELOG.zh.md
git commit -m "0.1.3"
git tag -a v0.1.3 -m "Release v0.1.3"
git push origin master
git push origin v0.1.3
```

## Short Checklist

For each release, go through this list in order:

- Update the version in `Cargo.toml`
- Update `CHANGELOG.md`
- Sync `CHANGELOG.zh.md`
- Check that the working tree is clean
- Create the release commit
- Create the `vX.Y.Z` tag
- Push the branch
- Push the tag
- Create the GitHub Release
- Verify the Release page contents
