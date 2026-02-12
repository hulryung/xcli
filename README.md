# xcli

X (Twitter) API CLI tool.

## Setup

### Prerequisites

[Rust](https://rustup.rs/)가 설치되어 있어야 합니다.

```bash
cargo build --release
```

### Authentication

두 가지 인증 방식을 지원합니다.

#### A. OAuth Login (사내 배포용)

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

#### B. Direct Token (개인 사용)

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
