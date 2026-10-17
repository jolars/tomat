# AUR package

`PKGBUILD` defines the `tomat-bin` package for x86_64 and aarch64. It installs
release binaries, man pages, shell completions, the systemd user service, and
the license.

After Build Release Binaries uploads all release assets, Publish AUR Package
downloads the two GNU/Linux archives, computes their SHA-256 checksums, tests
the package in Arch Linux, generates `.SRCINFO`, and pushes to AUR. Version and
checksum changes happen in the workflow checkout; they are not committed here.

Set the `AUR_SSH_PRIVATE_KEY` secret in `jolars/tomat` to a private SSH key whose
public key is registered with the AUR account that maintains `tomat-bin`.

To retry a publication, run Publish AUR Package manually with the release tag
(for example, `v2.13.0`). Set `pkgrel` above 1 when revising the package for an
existing version.

This replaces the dispatch workflow to `jolars/tomat-aur`. The separate
repository is no longer needed for release automation.
