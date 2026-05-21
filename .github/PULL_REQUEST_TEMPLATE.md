## Description

Brief description of changes made.

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)

## Checklist

### Code Quality
- [ ] My code follows the project's coding style (rustfmt, clippy clean)
- [ ] I have performed a self-review of my own code
- [ ] My changes generate no new warnings
- [ ] **No new `unwrap()` calls introduced** (CI will fail if detected)
  - If unavoidable, use `#[allow(clippy::unwrap_used)]` with justification comment

### Testing
- [ ] I have added tests that prove my fix is effective or my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] `cargo test --workspace` passes

### Documentation
- [ ] I have updated relevant documentation (README, CHANGELOG, doc comments)
- [ ] CHANGELOG.md updated with a summary of changes

## Error Handling Notes

If this PR modifies error handling:
- [ ] Uses `Result<T, E>` for fallible operations
- [ ] Uses `.expect("descriptive message")` only for truly impossible failures
- [ ] Uses `?` operator for error propagation
- [ ] No bare `.unwrap()` calls

## Related Issues

Fixes #(issue number)
