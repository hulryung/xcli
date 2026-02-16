# xcli

[![CI](https://github.com/hulryung/xcli/actions/workflows/ci.yml/badge.svg)](https://github.com/hulryung/xcli/actions/workflows/ci.yml)
[![Release](https://github.com/hulryung/xcli/actions/workflows/release.yml/badge.svg)](https://github.com/hulryung/xcli/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/hulryung/xcli)](https://github.com/hulryung/xcli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

X (Twitter) API CLI tool.

[한국어](README.ko.md)

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap hulryung/xcli
brew install xcli
```

### Binary Download

Download pre-built binaries from [GitHub Releases](https://github.com/hulryung/xcli/releases/latest).

| Platform | File |
|---|---|
| macOS (Intel) | `xcli-x86_64-apple-darwin.tar.gz` |
| macOS (Apple Silicon) | `xcli-aarch64-apple-darwin.tar.gz` |
| Linux (x86_64) | `xcli-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `xcli-aarch64-unknown-linux-musl.tar.gz` |
| Windows (x86_64) | `xcli-x86_64-pc-windows-msvc.zip` |
| Windows (ARM64) | `xcli-aarch64-pc-windows-msvc.zip` |

### Cargo

```bash
cargo install --git https://github.com/hulryung/xcli.git
```

### Build from Source

```bash
git clone https://github.com/hulryung/xcli.git
cd xcli
cargo build --release
```

## Authentication

Two authentication methods are supported.

### A. OAuth Login (Team Use)

An admin provides the X Developer App's API Key/Secret, and each user authenticates with their own account.

1. Set the admin-provided keys in a `.env` file:
   ```
   X_API_KEY=your_api_key
   X_API_SECRET=your_api_secret
   ```

2. Login:
   ```bash
   xcli auth login
   ```
   A browser will open for you to authorize the app with your X account. Tokens are saved to `~/.config/xcli/credentials.json`.

> **App Setup (Admin)**: Register `http://127.0.0.1:18923/callback` as a Callback URL in the X Developer Portal.

### B. Direct Token (Personal Use)

Create your own X Developer App and use it directly.

1. Create an app at the [X Developer Portal](https://developer.x.com)
2. Set all 4 tokens in a `.env` file:
   ```
   X_API_KEY=your_api_key
   X_API_SECRET=your_api_secret
   X_ACCESS_TOKEN=your_access_token
   X_ACCESS_TOKEN_SECRET=your_access_token_secret
   ```

No `auth login` required.

## Usage

### Post a Tweet

```bash
xcli tweet "Hello from xcli!"
# Tweet posted! ID: 1234567890
```

### Post a Thread

Long text is automatically split into a thread. You can also use `---` separators to control where splits occur.

```bash
# Auto-split long text into a thread
xcli tweet "First tweet content...
---
Second tweet content...
---
Third tweet content..."
# Thread posted! (3 tweets)
#   [1/3] ID: 1111111111
#   [2/3] ID: 2222222222
#   [3/3] ID: 3333333333

# Preview thread split without posting
xcli tweet "long text here..." --dry-run
# Thread preview (2 tweets):
#   [1/2] (250/280) First chunk...
#   [2/2] (180/280) Second chunk...
```

### Delete a Tweet

```bash
xcli delete 1234567890
# Tweet 1234567890 deleted.
```

### Manage Authentication

```bash
# OAuth login (opens browser)
xcli auth login
# Logged in as @username

# Check login status
xcli auth status
# Logged in as @username
# Credentials: /Users/you/.config/xcli/credentials.json

# Logout (remove stored credentials)
xcli auth logout
# Logged out. Credentials removed.
```

## Auth Priority

1. `~/.config/xcli/credentials.json` (tokens saved via OAuth login)
2. `X_ACCESS_TOKEN` / `X_ACCESS_TOKEN_SECRET` from `.env`

## License

[MIT](LICENSE)
