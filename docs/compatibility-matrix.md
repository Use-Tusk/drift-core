# Compatibility Matrix

Source of truth for native artifact coverage.

## Support Status

- Supported: release-blocking coverage in CI/publish workflow
- Best effort: no release-blocking guarantee; source-build or fallback paths may be required

## Python (`drift-core-python`)

| Platform | Artifact type | Support status |
| --- | --- | --- |
| Linux x86_64 (glibc) | wheel (`manylinux`) | Supported |
| Linux arm64 (glibc) | wheel (`manylinux`) | Supported |
| Linux x86_64 (musl) | wheel (`musllinux`) | Supported |
| Linux arm64 (musl) | wheel (`musllinux`) | Supported |
| macOS x86_64 (Intel) | wheel (`macosx`) | Supported |
| macOS arm64 (Apple Silicon) | wheel (`macosx`) | Supported |
| Windows x86_64 | wheel (`win_amd64`) | Supported |
| Windows arm64 | source build fallback | Best effort |

## Node (`@use-tusk/drift-core-node`)

| Platform | Artifact type | Support status |
| --- | --- | --- |
| Linux x86_64 (glibc) | prebuilt native addon | Supported |
| Linux arm64 (glibc) | prebuilt native addon | Supported |
| macOS x86_64 (Intel) | prebuilt native addon | Supported |
| macOS arm64 (Apple Silicon) | prebuilt native addon | Supported |
| Windows x86_64 | prebuilt native addon | Supported |
| Linux musl targets | no supported prebuild guarantee | Best effort |
| Windows arm64 | no supported prebuild guarantee | Best effort |

## Notes

- SDKs are fail-open. If a compatible native artifact is unavailable or fails to load, SDKs fall back to Python/JavaScript implementations.
- Runner labels can evolve over time (for example, macOS image deprecations), but this matrix reflects current intended support coverage.
