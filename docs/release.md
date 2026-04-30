# Release, signing, and notarization

This repo can produce local release artifacts without requiring a hosted
service. macOS signing and notarization use Apple credentials from the
developer's machine or CI secrets; credentials must never be committed.

## Current local macOS state

On this development Mac, the one-time notarization setup has already
been completed:

| Item | Value |
|------|-------|
| Team ID | `93T3LG4NPG` |
| Signing identity | `Developer ID Application: Fight The Stroke Foundation (93T3LG4NPG)` |
| notarytool profile | `convergio-notary` |
| Last accepted submission | `8466b59c-c2c1-406d-bf21-b8181d54cce2` |
| Last notarized artifact | `dist/convergio-darwin-arm64-signed.zip` |
| Last artifact SHA-256 | `e2c94fe4e2edbbd068cae2c84ba9c31e32129e49a2224f2cf246750c0f74c91d` |

The temporary Desktop setup helper can be deleted after the profile is
created because `notarytool` stores the credential in the macOS Keychain.

## Normal local release flow

After code changes, build, package, sign, and notarize with:

```bash
sh scripts/package-local.sh
APPLE_NOTARY_PROFILE=convergio-notary sh scripts/sign-macos-local.sh
```

This produces:

| File | Purpose |
|------|---------|
| `dist/convergio-darwin-arm64.tar.gz` | unsigned local tarball |
| `dist/convergio-darwin-arm64-signed.zip` | signed and notarized macOS zip |
| `dist/convergio-darwin-arm64-signed.zip.sha256` | checksum |

Verify the result:

```bash
for bin in dist/convergio-darwin-arm64/bin/convergio \
  dist/convergio-darwin-arm64/bin/cvg \
  dist/convergio-darwin-arm64/bin/convergio-mcp; do
  codesign --verify --strict --verbose=2 "$bin"
done

xcrun notarytool log <submission-id> --keychain-profile convergio-notary
```

## One-time notarization setup

Only repeat this if the Keychain profile is missing, expired, or created
for the wrong Apple ID:

```bash
xcrun notarytool store-credentials convergio-notary \
  --apple-id "<apple-id-in-team>" \
  --team-id "93T3LG4NPG"
```

Use an Apple **app-specific password**, not the normal iCloud password
and not a 2FA code. The Apple ID must belong to the developer team.

## CI release workflow

`.github/workflows/release.yml` runs fmt, clippy, tests, `cargo deny`,
and `cargo audit` before building unsigned Linux and macOS tarballs on
release tags. Each release artifact is paired with an SPDX JSON SBOM,
SHA-256 checksums, and a GitHub build-provenance attestation created with
OIDC. These checks do not require repository secrets.

To notarize in CI later, add GitHub secrets for either:

| Secret | Meaning |
|--------|---------|
| `APPLE_API_KEY_PATH` or `.p8` content secret | App Store Connect API key |
| `APPLE_API_KEY_ID` | API key ID |
| `APPLE_API_ISSUER_ID` | issuer UUID |
| `APPLE_SIGNING_CERTIFICATE_P12` | Developer ID Application certificate |
| `APPLE_SIGNING_CERTIFICATE_PASSWORD` | certificate password |

Do not fake signing or notarization in CI. If credentials are absent,
publish unsigned artifacts and label them as unsigned.

## Local supply-chain checks

Local development does not require supply-chain tools unless you want to
preflight CI. Optional commands:

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo deny --locked check advisories bans licenses sources
cargo audit
```

`deny.toml` owns dependency source, license, ban, and RustSec advisory
policy. `.cargo/audit.toml` makes `cargo audit` fail on vulnerabilities,
unsound/unmaintained informational advisories, and yanked crates. SBOMs and
GitHub provenance are release workflow outputs; they are not a substitute
for future capability package signatures.
