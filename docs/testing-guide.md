# Testing GitSwitch

## 🧪 Running the Tests

I've built a comprehensive test suite that makes sure GitSwitch works perfectly on Linux, Windows, and macOS. Here's how to run it:

```bash
# Run all tests
cargo test

# Run with colorful output (my favorite!)
cargo test -- --color=always

# Run specific test categories
cargo test account_management
cargo test repository_operations
cargo test ssh_validation
```

## 🎨 What Makes These Tests Special

### Colorful, Clear Output

I've made the test output beautiful and informative:

- 🟢 **Green** for successful operations
- � **Red** for errors and failures
- 🔵 **Blue** for informational messages
- 🟡 **Yellow** for warnings
- ✅ Clear success indicators
- 📊 Progress tracking for complex operations

### Cross-Platform Testing

Every test works on:

- **Linux** (Ubuntu, RHEL, etc.)
- **Windows** (PowerShell, Git Bash)
- **macOS** (Intel and Apple Silicon)

### Real-World Scenarios

I test actual use cases developers face:

- Multiple Git accounts (work/personal)
- SSH key management and validation
- Repository-specific configurations
- Template-based account creation
- Remote URL conversions (HTTPS ↔ SSH)

## �️ Test Categories

### Account Management Tests (8 tests)

- Creating accounts with different configurations
- Listing accounts (empty state, simple, detailed views)
- Switching between accounts globally
- Removing accounts safely
- Handling special characters and spaces in names

### Repository Operations Tests (4 tests)

- Applying accounts to specific repositories
- Converting remote URLs between HTTPS and SSH
- Identity verification (`whoami` command)
- Local repository configuration

### SSH Key Validation Tests (6 tests)

- OpenSSH private key validation
- Traditional private key format support
- SSH public key validation with key type detection
- Key strength recommendations
- Key pair verification using ssh-keygen
- Base64 validation for key data

### Template System Tests (2 tests)

- Listing available account templates
- Creating accounts from templates (GitHub, GitLab, Bitbucket)

### Profile Management Tests (6 tests)

- Creating and managing profiles
- Grouping accounts into workflows
- Profile statistics and usage tracking
- Bulk operations on profiles

### Repository Discovery Tests (4 tests)

- Recursive Git repository discovery
- Bulk account application with dry-run
- Interactive repository configuration
- Markdown report generation

#### **Authentication (1 test)**

- ✅ `test_auth_test_command` - SSH authentication testing

## 🧪 How I Test Everything

### Isolated Test Environments

Each test runs in its own clean environment:

- Temporary directories for each test
- Fresh Git configurations with no system interference
- Isolated SSH directories and configurations
- Platform-specific environment variables

### Comprehensive Error Testing

I test all the ways things can go wrong:

- Duplicate account prevention
- Missing account error handling
- Email validation errors
- Invalid SSH key handling
- Non-existent repository scenarios

### Real-World Workflows

I test complete scenarios developers actually use:

- Setting up work and personal accounts
- Switching between accounts for different projects
- Converting repositories between HTTPS and SSH
- Bulk operations on multiple repositories
- Template-based account creation

## 📊 Test Results

- **31 comprehensive tests** covering all features
- **100% command coverage** - every CLI command tested
- **Cross-platform support** - Linux, Windows, macOS
- **Real-world scenarios** - actual developer workflows
- **Beautiful output** - colorful, informative test results

## 🚀 Running Specific Tests

```bash
# Test account management
cargo test account_management

# Test SSH key validation
cargo test ssh_validation

# Test repository operations
cargo test repository_operations

# Test with verbose output
cargo test -- --nocapture
```

## 💡 Why I Built This Testing Framework

I wanted to make sure GitSwitch works perfectly for every developer, on every platform, in every scenario. The test suite gives me confidence that:

- All features work as expected
- Cross-platform compatibility is maintained
- Error handling is comprehensive and helpful
- Real-world workflows are supported
- The code is maintainable and extensible

When you run `cargo test`, you're seeing the result of careful testing that ensures GitSwitch will work reliably for your Git identity management needs! 🎯

- **New command testing** by following established patterns
- **Additional platform support** through helper function updates
- **Performance benchmarking** with timing assertions
- **Integration with external services** for SSH testing
- **Stress testing** with large numbers of accounts

This comprehensive test suite ensures GitSwitch works reliably across all platforms and provides confidence in every feature and command available to users.
