# Changelog

## [Unreleased]
### Fixed
- [JQ-1] `cancel_job()` TOCTOU race condition: replaced two-step SELECT+UPDATE with atomic UPDATE+affected check
- [PATH-1] Path dependency normalized from `../../Libraries/stack-ids` to `../stack-ids`
### Added
- Test for cancel-after-complete returning accurate status
