# Contributing to Xplainit Framework

Thank you for your interest in contributing to Xplainit! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions with the community.

## Getting Started

1. **Fork the repository** and clone it locally
2. **Set up your development environment**:
   ```bash
   cd xplainit-framework
   cargo build
   cargo test
   ```

3. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Workflow

### Building

```bash
# Build all packages
cargo build

# Build in release mode
cargo build --release

# Build specific package
cargo build -p xplainit-core
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p xplainit-core

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all

# Check documentation
cargo doc --no-deps
```

## Contribution Guidelines

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Write clear, self-documenting code
- Add comments for complex logic
- Use meaningful variable and function names

### Documentation

- Add doc comments (`///`) for all public APIs
- Include examples in doc comments where helpful
- Update README.md if adding new features
- Add inline comments for complex algorithms

### Testing

- Write unit tests for new functionality
- Add integration tests for multi-component features
- Ensure all tests pass before submitting PR
- Aim for high test coverage (>80%)
- Test edge cases and error conditions

### Commit Messages

Use clear, descriptive commit messages:

```
feat: Add support for custom event filters
fix: Resolve memory leak in event store
docs: Update README with new examples
test: Add integration tests for error analysis
perf: Optimize event pipeline throughput
```

Prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or modifications
- `perf:` - Performance improvements
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

### Pull Request Process

1. **Update documentation** for any changed functionality
2. **Add tests** for new features
3. **Ensure all CI checks pass**
4. **Write a clear PR description** explaining:
   - What changes were made
   - Why the changes were necessary
   - How to test the changes
5. **Link related issues** using `Fixes #123` or `Relates to #456`
6. **Request review** from maintainers

### PR Review Checklist

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] Changes are focused and atomic

## Project Structure

```
xplainit-framework/
├── xplainit-core/          # Core Rust library
│   ├── src/                # Source code
│   ├── tests/              # Integration tests
│   ├── examples/           # Usage examples
│   └── benches/            # Benchmarks
├── xplainit-python/        # Python bindings
├── xplainit-js/            # JavaScript bindings
└── docs/                   # Additional documentation
```

## Areas for Contribution

### High Priority
- Language runtime integrations (Python, JavaScript, Java, etc.)
- Performance optimizations
- Additional event types
- Enhanced error analysis
- Documentation improvements

### Good First Issues
- Add more examples
- Improve error messages
- Add unit tests
- Fix typos in documentation
- Add inline code comments

### Advanced Topics
- Custom filter implementations
- New output formatters
- Advanced analysis algorithms
- Cross-language debugging features

## Language Bindings

When contributing language bindings:

1. Follow the language's conventions and idioms
2. Provide comprehensive examples
3. Write tests in the target language
4. Document installation and setup
5. Ensure compatibility with common versions

## Performance Considerations

- Profile code before and after changes
- Run benchmarks to measure impact
- Avoid unnecessary allocations
- Consider memory usage
- Optimize hot paths

## Questions and Help

- **Issues**: Open an issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the docs/ directory for guides

## License

By contributing to Xplainit, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be acknowledged in:
- CHANGELOG.md for each release
- Project README.md
- Release notes

Thank you for contributing to Xplainit! 🚀
