# CI Failure Prevention Guide

This guide explains how to prevent CI failures and maintain code quality consistently.

## The Problem

CI failures waste time and break the development flow. Common causes:
- **Formatting issues**: Code not formatted with `cargo fmt`
- **Clippy warnings**: Linting issues that fail with `-D warnings`
- **Test failures**: Broken tests that weren't caught locally
- **Documentation issues**: Missing docs or rustdoc warnings

## Prevention Strategy

### 1. Development Workflow Scripts

We provide two scripts to prevent CI failures:

#### Quick Check (During Development)
```bash
./scripts/quick-check.sh
```
- Auto-formats code with `cargo fmt --all`
- Runs quick compile check
- Runs all tests
- **Use frequently** during development

#### Pre-Commit Check (Before Committing)
```bash
./scripts/pre-commit-check.sh
```
- Runs **exact same checks as CI**
- Formatting, clippy, tests, documentation
- **ALWAYS run before committing**
- Only commit if this passes

### 2. Editor Integration

#### VS Code Settings
The `.vscode/settings.json` configures:
- **Format on save**: Automatically formats code
- **Clippy integration**: Shows warnings in editor
- **Rust analyzer**: Proper rust tooling setup

#### Other Editors
- **Vim/Neovim**: Install rust.vim + ale/coc
- **Emacs**: Use rustic-mode
- **IntelliJ**: Rust plugin with format on save

### 3. Git Hooks (Optional)

To enforce checks automatically:

```bash
# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
exec ./scripts/pre-commit-check.sh
EOF
chmod +x .git/hooks/pre-commit
```

### 4. Development Rules

**Golden Rules:**
1. **Never commit without pre-commit checks**
2. **Format code frequently** (`cargo fmt --all`)
3. **Run tests locally** before pushing
4. **Check clippy warnings** regularly

**Workflow:**
1. Make changes
2. Run `./scripts/quick-check.sh` frequently
3. Before committing: `./scripts/pre-commit-check.sh`
4. Only commit if all checks pass

### 5. CI Understanding

Our CI runs these jobs:
- **check (stable)**: Format, clippy, tests, docs
- **check (nightly)**: Same checks on nightly Rust
- **coverage**: Test coverage with 65% minimum
- **build-release**: Release build validation

**All jobs must pass** for merge to main.

## Troubleshooting

### Format Failures
```bash
# Fix: Run formatter
cargo fmt --all

# Check: Verify formatting
cargo fmt --all -- --check
```

### Clippy Failures
```bash
# Fix: Address warnings
cargo clippy --workspace --all-targets --all-features --fix

# Check: Verify no warnings
RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Test Failures
```bash
# Run tests with same flags as CI
RUSTFLAGS="-Dwarnings" cargo test --workspace --all-features

# Run specific test
cargo test --workspace --all-features test_name
```

### Documentation Failures
```bash
# Check documentation builds
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# Fix missing documentation
# Add /// comments to public items
```

## Integration with Oracle Strategy

This prevention strategy aligns with Oracle's Week 2 quality gates:
- **Coverage minimum**: 65% (will increase to 70%)
- **Rustdoc linting**: Enforced in CI
- **Clean builds**: No warnings allowed
- **Fast iteration**: Quick local validation

## Automation

Consider setting up:
- **Pre-commit hooks**: Automatic validation
- **IDE integration**: Real-time feedback
- **Git aliases**: Shortcuts for common commands
- **Local CI**: Run full CI pipeline locally

The goal is to **catch issues early** and maintain high code quality without slowing down development.
