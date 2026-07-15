# Changelog

All notable changes to this project will be documented in this file, structured by version with their corresponding GitHub Pull Requests and commit hashes.

## [Unreleased]

- **fix(ci)**: grant workflow write permissions and add self-healing hash detection (#146) [[c37ffc8](https://github.com/Spectra010s/portal/commit/c37ffc8)]
- **refactor(docs)**: Use native tabs instead of custom demo components (#141) [[39644b3](https://github.com/Spectra010s/portal/commit/39644b3)]
- **fix(web)**: Add required text property to GitHub icon link (#139) [[c9c7ec8](https://github.com/Spectra010s/portal/commit/c9c7ec8)]
- **refactor(web)**: Migrate documentation framework from Nextra to Fumadocs (#138) [[d38b4f8](https://github.com/Spectra010s/portal/commit/d38b4f8)]
- **chore(config)**: Consolidate repository ignore rules [[68a4fc7](https://github.com/Spectra010s/portal/commit/68a4fc7)]
- **build(web)**: Switch web docs application to pnpm workspace [[24f3f8e](https://github.com/Spectra010s/portal/commit/24f3f8e)]
- **fix(docs)**: Contain mobile code block layout overflow (#132) [[6934b93](https://github.com/Spectra010s/portal/commit/6934b93)]

---

## [v0.12.0] - 2026-06-11

- **feat(discovery)**: Add broadcast fallback when multicast discovery fails (#123) [[2ad240c](https://github.com/Spectra010s/portal/commit/2ad240c)]
- **fix(receiver)**: Keep receive mode running without local IP (#125) [[0bacece](https://github.com/Spectra010s/portal/commit/0bacece)]
- **fix(web)**: Correct portal domain in install links (#116) [[3f1a985](https://github.com/Spectra010s/portal/commit/3f1a985)]
- **ci(release)**: Replace cargo-dist workflow (#119) [[80071d0](https://github.com/Spectra010s/portal/commit/80071d0)]
- **ci(release)**: Fix cargo-wix source input (#127) [[4d2e0bb](https://github.com/Spectra010s/portal/commit/4d2e0bb)]
- **ci(release)**: Use available macos intel runner (#128) [[1b73b8d](https://github.com/Spectra010s/portal/commit/1b73b8d)]
- **ci(release)**: Checkout before creating release (#129) [[7c43d29](https://github.com/Spectra010s/portal/commit/7c43d29)]
- **chore(config)**: Add git-aic prompt configuration (#120) [[9a09c6a](https://github.com/Spectra010s/portal/commit/9a09c6a)]
- **docs(repo)**: Add contributor and support guides (#121) [[4f6cec3](https://github.com/Spectra010s/portal/commit/4f6cec3)]
- **chore(release)**: Bump version to 0.12.0 (#126) [[fcc504e](https://github.com/Spectra010s/portal/commit/fcc504e)]

---

## [v0.11.1] - 2026-05-31

- **build(deps)**: Bump tar from 0.4.45 to 0.4.46 (#115) [[519c62c](https://github.com/Spectra010s/portal/commit/519c62c)]
- **build(deps)**: Bump astral-tokio-tar from 0.6.0 to 0.6.2 (#114) [[bb18b66](https://github.com/Spectra010s/portal/commit/bb18b66)]
- **build(deps)**: Bump next from 16.2.4 to 16.2.6 in /apps/web (#112) [[10cba9b](https://github.com/Spectra010s/portal/commit/10cba9b)]
- **build(deps)**: Bump uuid and mermaid in /apps/web (#113) [[e97a1c8](https://github.com/Spectra010s/portal/commit/e97a1c8)]
- **build(deps)**: Bump brace-expansion in /apps/web (#102) [[385c005](https://github.com/Spectra010s/portal/commit/385c005)]
- **build(deps)**: Bump lodash-es and langium in /apps/web (#101) [[a34311e](https://github.com/Spectra010s/portal/commit/a34311e)]
- **build(deps)**: Bump picomatch in /apps/web (#100) [[7280407](https://github.com/Spectra010s/portal/commit/7280407)]
- **chore(release)**: Bump version to 0.11.1 [[d30bf48](https://github.com/Spectra010s/portal/commit/d30bf48)]

---

## [v0.11.0] - 2026-05-03

- **feat(logging)**: Add global verbose and quiet flags [[0fadc65](https://github.com/Spectra010s/portal/commit/0fadc65)]
- **feat(logging)**: Add `PORTAL_LOG` with `RUST_LOG` fallback [[b5d8613](https://github.com/Spectra010s/portal/commit/b5d8613)]
- **feat(logging)**: Split terminal and file log levels [[e4d8241](https://github.com/Spectra010s/portal/commit/e4d8241)]
- **feat(updater)**: Add shared updater download progress UI [[f4ebb52](https://github.com/Spectra010s/portal/commit/f4ebb52)]
- **chore(docs)**: Remove duplicated docs directory and use web docs source (#105) [[785a2f6](https://github.com/Spectra010s/portal/commit/785a2f6)]
- **docs(logging)**: Document verbosity flags and PORTAL_LOG precedence [[fbce61f](https://github.com/Spectra010s/portal/commit/fbce61f)]
- **build(deps)**: Bump dompurify from 3.3.3 to 3.4.1 in /apps/web (#98) [[35dd1a6](https://github.com/Spectra010s/portal/commit/35dd1a6)]
- **build(deps)**: Bump next from 16.2.1 to 16.2.4 in /apps/web (#95) [[eef65f9](https://github.com/Spectra010s/portal/commit/eef65f9)]
- **build(deps)**: Bump rustls-webpki from 0.103.9 to 0.103.13 (#97) [[69c471e](https://github.com/Spectra010s/portal/commit/69c471e)]
- **build(deps)**: Bump rand from 0.10.0 to 0.10.1 (#96) [[749a546](https://github.com/Spectra010s/portal/commit/749a546)]
- **build(deps)**: Bump @xmldom/xmldom and speech-rule-engine in /apps/web (#99) [[f7d612f](https://github.com/Spectra010s/portal/commit/f7d612f)]
- **chore(release)**: Bump version to 0.11.0 [[42dd453](https://github.com/Spectra010s/portal/commit/42dd453)]

---

## [v0.10.1] - 2026-03-22

- **refactor(project)**: Flatten `hiverra-portal` and archive old web portals (#90) [[d53b173](https://github.com/Spectra010s/portal/commit/d53b173)]
- **chore(deps)**: Remove confy crate (#89) [[7ff27c2](https://github.com/Spectra010s/portal/commit/7ff27c2)]
- **refactor(portal)**: Improve code formatting and async error handling (#88) [[9a0d509](https://github.com/Spectra010s/portal/commit/9a0d509)]
- **build(deps)**: Bump tar from 0.4.44 to 0.4.45 in /hiverra-portal (#87) [[a765ac4](https://github.com/Spectra010s/portal/commit/a765ac4)]
- **fix(npm-publish)**: Improve archive extraction and release notes [[bac7790](https://github.com/Spectra010s/portal/commit/bac7790)]
- **ci(npm)**: Implement dedicated package publish workflow (#86) [[7850321](https://github.com/Spectra010s/portal/commit/7850321)]
- **chore(project)**: Bump version to 0.10.1 [[58fa336](https://github.com/Spectra010s/portal/commit/58fa336)]

---

## [v0.10.0] - 2026-03-20

- **feat(transfer)**: Support directory transfers and fix tar/gzip streaming errors (#49) [[ede3d2e](https://github.com/Spectra010s/portal/commit/ede3d2e)]
- **feat(transfer)**: Add support for uncompressed transfers (`--no-compress`) (#74) [[a957f2a](https://github.com/Spectra010s/portal/commit/a957f2a)]
- **feat(history)**: Add comprehensive transfer history tracking system (#62) [[a914473](https://github.com/Spectra010s/portal/commit/a914473)]
- **feat(history)**: Add export command and consolidate filters (#72) [[245508b](https://github.com/Spectra010s/portal/commit/245508b)]
- **feat(history)**: Add clear and delete actions (#71) [[1b9046c](https://github.com/Spectra010s/portal/commit/1b9046c)]
- **feat(receiver)**: Record partial summary on transfer error (#78) [[fadb8f3](https://github.com/Spectra010s/portal/commit/fadb8f3)]
- **refactor(portal)**: Organize receiver and sender modules (#76) [[2ba68ce](https://github.com/Spectra010s/portal/commit/2ba68ce)]
- **refactor(release)**: Scope npm package as `@hiverra/portal` (#84) [[c13013a](https://github.com/Spectra010s/portal/commit/c13013a)]
- **docs**: Add install, usage, and index pages (#82) [[d3ff187](https://github.com/Spectra010s/portal/commit/d3ff187)]
- **chore(release)**: Bump version to 0.10.0 and mark roadmap items complete (#85) [[630ac92](https://github.com/Spectra010s/portal/commit/630ac92)]

---

## [v0.9.0] - 2026-02-19

- **feat(transfer)**: Support sending multiple files with transfer manifest (#41) [[7a31cf3](https://github.com/Spectra010s/portal/commit/7a31cf3)]
- **feat(discovery)**: Implement local discovery protocol and identity handshake [[e08aa25](https://github.com/Spectra010s/portal/commit/e08aa25)]
- **feat(config)**: Refactor PortalConfig to nested architecture and interactive init (#34) [[fa16824](https://github.com/Spectra010s/portal/commit/fa16824)]
- **fix(updater)**: Correct binary replacement for all platforms [[a90f6e4](https://github.com/Spectra010s/portal/commit/a90f6e4)]
- **fix**: Add XXXXXX for the TMP directory (#44) [[bafe5f3](https://github.com/Spectra010s/portal/commit/bafe5f3)]
- **hotfix**: Fix installer script TMP_DIR/ order (#45) [[26e4aa2](https://github.com/Spectra010s/portal/commit/26e4aa2)]

---

## [v0.8.0] - 2026-02-12

- **feat(config)**: Implement configuration system and inquire wizard [[d433452](https://github.com/Spectra010s/portal/commit/d433452)]

---

## [v0.7.0] - 2026-02-12

- **feat(commands)**: Add custom `--port` flag support for send and receive [[e851dc5](https://github.com/Spectra010s/portal/commit/e851dc5)]

---

## [v0.6.1-rc.3] - 2026-02-12

- **fix(ci)**: Stabilize Android section & table row insertion in release body [[772d47b](https://github.com/Spectra010s/portal/commit/772d47b)]

---

## [v0.6.1-rc.2] - 2026-02-12

- **fix**: Replace `rustls-tls` with `rustls` in reqwest [[c79e27c](https://github.com/Spectra010s/portal/commit/c79e27c)]

---

## [v0.6.1-rc.1] - 2026-02-11

*(Release candidate baseline)*

---

## [v0.6.1-beta.1] - 2026-02-12

- **fix(ci)**: Stabilize Android section & table row insertion in release body [[cf45a8f](https://github.com/Spectra010s/portal/commit/cf45a8f)]
- **fix**: Replace `rustls-tls` with `rustls` in reqwest [[c79e27c](https://github.com/Spectra010s/portal/commit/c79e27c)]
- **fix**: Improve self-update process for all platforms [[43d5e40](https://github.com/Spectra010s/portal/commit/43d5e40)]

---

## [v0.6.0-rc.1] - 2026-02-11

- **feat**: Integrate inquire for interactive user flow (#21) [[7f8db57](https://github.com/Spectra010s/portal/commit/7f8db57)]
- **fix(android-release)**: Update installer script creation [[6e50d84](https://github.com/Spectra010s/portal/commit/6e50d84)]
- **fix**: Indentation error in android release yml [[0b26803](https://github.com/Spectra010s/portal/commit/0b26803)]

---

## [v0.5.1-beta.2] - 2026-02-11

- **chore**: Remove aarch64-pc-windows-msvc build target [[39db122](https://github.com/Spectra010s/portal/commit/39db122)]
- **fix**: Update get_local_ip to use interface name filtering (#14) [[3db7fb8](https://github.com/Spectra010s/portal/commit/3db7fb8)]
- **refactor**: Integrate tokio and add update command (#19) [[1cfb85e](https://github.com/Spectra010s/portal/commit/1cfb85e)]
- **fix**: Resolve type mismatches and unused imports in update logic (#20) [[1f54dac](https://github.com/Spectra010s/portal/commit/1f54dac)]

---

## [v0.5.1-beta.1] - 2026-02-11

- **refactor**: Clean up redundant imports and unnecessary mutability [[c0b682b](https://github.com/Spectra010s/portal/commit/c0b682b)]

---

## [v0.5.1-beta] - 2026-02-11

- **fix**: Resolve Android linker errors and Windows unreachable code [[f088297](https://github.com/Spectra010s/portal/commit/f088297)]

---

## [v0.5.1-alpha.5] - 2026-02-11

- **fix**: Purge openssl-sys by disabling default-features for self_update [[4093d61](https://github.com/Spectra010s/portal/commit/4093d61)]

---

## [v0.5.1-alpha.4] - 2026-02-11

- **fix**: Use rustls to fix Android cross-compilation [[7b32590](https://github.com/Spectra010s/portal/commit/7b32590)]

---

## [v0.5.1-alpha.3] - 2026-02-11

- **fix(ci)**: Add env after run so as to use in environment [[160124b](https://github.com/Spectra010s/portal/commit/160124b)]

---

## [v0.5.1-alpha.2] - 2026-02-11

- **fix(ci)**: Enable vendored OpenSSL for Android build [[a4b6b1d](https://github.com/Spectra010s/portal/commit/a4b6b1d)]

---

## [v0.5.1-alpha.1] - 2026-02-11

- **fix(android)**: Use --platform for API level and remove manual NDK install [[3f9f30f](https://github.com/Spectra010s/portal/commit/3f9f30f)]

---

## [v0.5.1-alpha] - 2026-02-11

- **fix**: Resolve type mismatches and unused imports in update logic [[1f54dac](https://github.com/Spectra010s/portal/commit/1f54dac)]
- **fix(android-installer)**: Replace command checks with direct pkg install for required deps [[2d90bf6](https://github.com/Spectra010s/portal/commit/2d90bf6)]

---

## [v0.5.0-alpha] - 2026-02-11

- **feat(ci)**: Add Android release workflow [[bc051c7](https://github.com/Spectra010s/portal/commit/bc051c7)]
- **refactor**: Integrate tokio and add update command (#6) (#17) [[1cfb85e](https://github.com/Spectra010s/portal/commit/1cfb85e)]
- **feat**: Add GH ISSUE_TEMPLATES for bug and feature [[68f5edd](https://github.com/Spectra010s/portal/commit/68f5edd)]
- **fix**: Indentation error in yml for android release [[f2035de](https://github.com/Spectra010s/portal/commit/f2035de)]

---

## [v0.4.0-beta.2] - 2026-02-09

- **fix**: Update get_local_ip to use interface name filtering [[3db7fb8](https://github.com/Spectra010s/portal/commit/3db7fb8)]
- **build**: Add android artifact packaging script [[e6d380c](https://github.com/Spectra010s/portal/commit/e6d380c)]

---

## [v0.4.0-beta.1] - 2026-02-06

- **fix(dist)**: Remove aarch64-pc-windows-msvc from targets [[2d978b0](https://github.com/Spectra010s/portal/commit/2d978b0)]
- **fix(dist)**: Move `dist-workspace.toml` to repo root [[62b7410](https://github.com/Spectra010s/portal/commit/62b7410)]
- **fix(ci)**: Regenerate release workflow and correct prerelease tag [[04857c2](https://github.com/Spectra010s/portal/commit/04857c2)]

---

## [v0.4.0-beta] - 2026-02-06

- **feat**: Implement global networking and automatic IP discovery (#12) [[dcf696c](https://github.com/Spectra010s/portal/commit/dcf696c)]
- **feat**: Add file descriptions and implement metadata serialization (#11) [[fec1c38](https://github.com/Spectra010s/portal/commit/fec1c38)]
- **build**: Initialize cargo-dist v0.4.0 and cross-compile targets [[8773d67](https://github.com/Spectra010s/portal/commit/8773d67)]
- **fix(ci)**: Move defaults.run to workflow level and correct tag trigger [[e1a80e7](https://github.com/Spectra010s/portal/commit/e1a80e7)]

---

## [v0.3.0] - 2026-02-05

- **feat**: Scaffold repo and setup rust folder for portal engine (#1) [[b8a7b65](https://github.com/Spectra010s/portal/commit/b8a7b65)]
- **feat**: Implement TCP file transfer protocol with metadata handshaking [[498b248](https://github.com/Spectra010s/portal/commit/498b248)]
- **feat**: Implement robust error handling with anyhow (#10) [[a4216f4](https://github.com/Spectra010s/portal/commit/a4216f4)]
- **refactor**: Modularize send and receive logic into dedicated files (#2) (#9) [[162ee27](https://github.com/Spectra010s/portal/commit/162ee27)]
- **chore**: Archive experimental TCP prototypes [[33b2eae](https://github.com/Spectra010s/portal/commit/33b2eae)]
- **docs**: Establish modular documentation system [[78fa0f5](https://github.com/Spectra010s/portal/commit/78fa0f5)]
- **docs**: Implement security-first quarantine strategy [[e132c6c](https://github.com/Spectra010s/portal/commit/e132c6c)]
- **docs**: Add licensing documentation [[3d79272](https://github.com/Spectra010s/portal/commit/3d79272)]
- **chore**: Update version to 0.2.0 and update manifest authors [[0e00b18](https://github.com/Spectra010s/portal/commit/0e00b18)]
- **test**: Implement TCP sandbox with sender and receiver binaries [[e0befeb](https://github.com/Spectra010s/portal/commit/e0befeb)]
- **feat**: Implement file opening and buffered reader [[f5e83d4](https://github.com/Spectra010s/portal/commit/f5e83d4)]
- **feat**: Use metadata to get file size in send command [[a4c8163](https://github.com/Spectra010s/portal/commit/a4c8163)]
- **refactor**: Move commands and logic to dedicated module [[d9b12c3](https://github.com/Spectra010s/portal/commit/d9b12c3)]
- **feat**: Add directory check to send command [[9f4427e](https://github.com/Spectra010s/portal/commit/9f4427e)]
- **feat**: Add MIT License to the project [[83de4e5](https://github.com/Spectra010s/portal/commit/83de4e5)]
- **chore**: Remove own supabase keys [[82d53c9](https://github.com/Spectra010s/portal/commit/82d53c9)]
- **feat**: Initialized repo, Add first files [[5e1488b](https://github.com/Spectra010s/portal/commit/5e1488b)]
