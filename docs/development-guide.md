# Contributing to GitSwitch

## 🚀 Getting Started

I'd love your help making GitSwitch even better! Here's how to get started with development:

```bash
# Clone the repository
git clone https://github.com/yourusername/GitSwitch.git
cd GitSwitch

# Run tests to make sure everything works
cargo test

# Build the project
cargo build

# Try it out!
./target/debug/git-switch --help
```

## 🎨 What Makes Development Fun

### Beautiful Test Output

I've made the test output colorful and informative so you can easily see what's happening:

- 🧪 **Clear test headers** with context and platform info
- 📋 **Step-by-step progress** showing what each test is doing
- 🔧 **Command visibility** so you can see exactly what's being tested
- ✅ **Success indicators** with clear completion messages
- 🚨 **Helpful error messages** when things go wrong

### Clean Code Structure

The codebase is organized into focused modules:

- `src/commands.rs` - All CLI command implementations
- `src/profiles.rs` - Profile management system
- `src/repository.rs` - Repository discovery and operations
- `src/validation.rs` - SSH key validation and security
- `src/ssh.rs` - SSH key management and utilities
- `src/config.rs` - Configuration management
- `src/templates.rs` - Account template system

## 🧪 Testing Your Changes

### Running Tests

```bash
# Run all tests with colorful output
cargo test -- --color=always --nocapture

# Run specific test categories
cargo test account_management
cargo test ssh_validation

# Run a specific test
cargo test test_add_account_basic
```

### Adding New Tests

When adding new features, please include tests! Follow the pattern in `tests/comprehensive_platform_tests.rs`:

1. Create isolated test environments
2. Add helpful step descriptions
3. Test both success and error cases
4. Ensure cross-platform compatibility

## 🔧 Development Workflow

### Before You Start

1. **Read the code** - Get familiar with the existing patterns
2. **Run the tests** - Make sure everything passes on your system
3. **Try the CLI** - Use GitSwitch yourself to understand the user experience

### Making Changes

1. **Create a branch** for your feature/fix
2. **Write tests first** (TDD approach)
3. **Implement your changes**
4. **Run tests frequently** - `cargo test` is your friend!
5. **Update documentation** if needed

### Before Submitting

1. **Format your code**: `cargo fmt`
2. **Check for issues**: `cargo clippy`
3. **Run all tests**: `cargo test`
4. **Test build scripts**: `./build.sh --tarball --skip-tests`

## 🏗️ Architecture Overview

### Core Components

- **CLI Parser** (`main.rs`) - Command-line interface using clap
- **Account Management** (`commands.rs`) - Core account operations
- **SSH Integration** (`ssh.rs`) - SSH key management and validation
- **Configuration** (`config.rs`) - Settings and data persistence
- **Profile System** (`profiles.rs`) - Advanced workflow management
- **Repository Tools** (`repository.rs`) - Git repository operations

### Key Design Principles

1. **Cross-platform compatibility** - Works on Linux, Windows, macOS
2. **User-friendly output** - Colorful, clear, helpful messages
3. **Safety first** - Comprehensive validation and error handling
4. **Extensible** - Easy to add new features and providers
5. **Well-tested** - Comprehensive test coverage

## 💡 Tips for Contributors

### Understanding GitSwitch

- **Try using it** - The best way to understand GitSwitch is to use it for your own Git identity management
- **Read the tests** - The test suite shows all the features and how they work
- **Look at the CLI help** - `git-switch --help` shows all available commands

### Common Development Tasks

- **Adding a new command** - Follow the pattern in `commands.rs`
- **Adding SSH key support** - Extend the validation in `validation.rs`
- **Improving error messages** - Check `error.rs` for error types
- **Adding new providers** - Look at the template system in `templates.rs`

### Testing Guidelines

- **Test cross-platform** - Make sure your changes work on different OSs
- **Test error cases** - Don't just test the happy path
- **Use descriptive test names** - Make it clear what each test does
- **Keep tests isolated** - Each test should be independent

## 🤝 Getting Help

If you have questions or need help:

1. **Check the documentation** - Start with the README and these docs
2. **Look at existing code** - See how similar features are implemented
3. **Run the tests** - They show how everything works
4. **Open an issue** - If you're stuck, ask for help!

I'm excited to see what improvements you'll bring to GitSwitch! 🎉
