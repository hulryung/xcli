# xcli

X (Twitter) API CLI tool.

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap hulryung/xcli
brew install xcli
```

### Winget (Windows)

```powershell
winget install hulryung.xcli
```

### Binary Download

[GitHub Releases](https://github.com/hulryung/xcli/releases/latest)에서 플랫폼별 바이너리를 다운로드할 수 있습니다.

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
cargo install xcli
```

### Build from Source

```bash
git clone https://github.com/hulryung/xcli.git
cd xcli
cargo build --release
```

## Authentication

두 가지 인증 방식을 지원합니다.

### A. OAuth Login (사내 배포용)

관리자가 X Developer App의 API Key/Secret을 제공하면, 각 사용자가 본인 계정으로 인증합니다.

1. `.env` 파일에 관리자가 제공한 키 설정:
   ```
   X_API_KEY=your_api_key
   X_API_SECRET=your_api_secret
   ```

2. 로그인:
   ```bash
   xcli auth login
   ```
   브라우저가 열리면 X 계정으로 앱을 승인합니다. 토큰이 `~/.config/xcli/credentials.json`에 저장됩니다.

> **App 설정 (관리자)**: X Developer Portal에서 Callback URL에 `http://127.0.0.1:18923/callback`을 등록해야 합니다.

### B. Direct Token (개인 사용)

본인이 X Developer App을 직접 만들어 사용합니다.

1. [X Developer Portal](https://developer.x.com)에서 앱 생성
2. `.env` 파일에 4개 토큰 설정:
   ```
   X_API_KEY=your_api_key
   X_API_SECRET=your_api_secret
   X_ACCESS_TOKEN=your_access_token
   X_ACCESS_TOKEN_SECRET=your_access_token_secret
   ```

`auth login` 없이 바로 사용 가능합니다.

## Usage

```bash
# Post a tweet
xcli tweet "Hello from xcli!"

# Delete a tweet
xcli delete <tweet_id>

# Check login status
xcli auth status

# Logout (remove stored credentials)
xcli auth logout
```

## Auth Priority

1. `~/.config/xcli/credentials.json` (OAuth login으로 저장된 토큰)
2. `.env`의 `X_ACCESS_TOKEN` / `X_ACCESS_TOKEN_SECRET`

## License

[MIT](LICENSE)
