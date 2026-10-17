# Export checks
- New history, not a copy of the private .git directory.
- Only Rust source, selected WPF editor scripts, Cargo files, neutral generated PNG and documentation are included.
- No character artwork, preview images, logs, runtime settings, compiled binaries or private sensor snapshots.
- Source search for credentials, private network addresses, personal paths and unrelated integrations reviewed.
- Global key binding implementation and associated configuration are absent.
- cargo test --locked: eight tests passed on Windows before publication.
- Hardware connection not exercised during export checks. Source tests do not establish support for every panel or sensor.
