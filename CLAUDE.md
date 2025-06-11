# Protowirers Development Rules

## Important Development Rules

### 1. Pre-change Checklist
- **Always run lint and tests before making any changes**
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo test --all-features`

### 2. Test-First Principle
- **Always create sufficient tests before making functional changes or optimizations**
- Verify that existing tests cover the changes
- If tests are insufficient, add tests first before changing implementation

### 3. Incremental Changes and Commits
- Commit changes with smaller impact first
- Always run lint and tests before each commit
- Split large changes into multiple smaller commits
