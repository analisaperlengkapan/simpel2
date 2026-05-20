## Description

<!-- Provide a brief description of the changes in this PR -->

## Type of Change

<!-- Mark the relevant option with an "x" -->

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📝 Documentation update
- [ ] 🔧 Configuration change
- [ ] ♻️ Code refactoring (no functional changes)
- [ ] ⚡ Performance improvement
- [ ] 🔒 Security fix
- [ ] 🧪 Test improvements

## Related Issues

<!-- Link related issues here using #issue_number -->

Closes #
Related to #

## Changes Made

<!-- Provide a detailed list of changes -->

-
-
-

## Component Impact

<!-- Mark which components are affected -->

- [ ] Backend Services (`layanan/`)
- [ ] Frontend Microfrontends (`antarmuka/`)
- [ ] API Gateway (`infra/gerbang/`)
- [ ] Secreton Vault (`infra/secreton/`)
- [ ] Shared Libraries
- [ ] Infrastructure/DevOps
- [ ] Documentation

## Testing

<!-- Describe the tests you ran and how to reproduce them -->

### Test Environment

- [ ] Local development
- [ ] Staging environment
- [ ] CI/CD pipeline

### Test Coverage

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] E2E tests added/updated
- [ ] Manual testing performed

### Test Results

```bash
# Paste test output here
cargo test --all
```

## Checklist

<!-- Mark completed items with an "x" -->

### Code Quality

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings
- [ ] I have run `cargo clippy --all -- -D warnings` with no errors
- [ ] I have run `cargo fmt --all` to format the code

### Testing

- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have run `cargo test --all` successfully

### Documentation

- [ ] I have updated the relevant documentation (README, docs/, comments)
- [ ] I have updated the CHANGELOG.md (if applicable)
- [ ] I have added/updated API documentation (if applicable)

### Security

- [ ] My changes do not introduce security vulnerabilities
- [ ] I have run `cargo audit` with no critical issues
- [ ] Sensitive data is properly handled (no credentials in code)
- [ ] Authentication/authorization checks are properly implemented (if applicable)

### Database

- [ ] Database migrations are included (if applicable)
- [ ] Database schema changes are backward compatible (if applicable)
- [ ] Migration scripts have been tested

### Deployment

- [ ] My changes are backward compatible
- [ ] Configuration changes are documented
- [ ] Environment variables are documented (if new ones added)
- [ ] Docker/K8s manifests are updated (if applicable)

## Performance Impact

<!-- Describe any performance implications -->

- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance degraded (explain below)

**Performance notes:**

## Breaking Changes

<!-- If this is a breaking change, describe the impact and migration path -->

**Migration guide:**

## Screenshots/Videos

<!-- If applicable, add screenshots or videos to demonstrate the changes -->

## Additional Notes

<!-- Add any additional notes for reviewers -->

## Reviewer Notes

<!-- For reviewers: Add your review comments here -->

---

**By submitting this PR, I confirm that:**

- [ ] I have read and followed the [Contributing Guidelines](../CONTRIBUTING.md)
- [ ] My code adheres to the project's coding standards
- [ ] I have tested my changes thoroughly
- [ ] I am authorized to submit this code under the project's license
