# xcli

[![CI](https://github.com/hulryung/xcli/actions/workflows/ci.yml/badge.svg)](https://github.com/hulryung/xcli/actions/workflows/ci.yml)
[![Release](https://github.com/hulryung/xcli/actions/workflows/release.yml/badge.svg)](https://github.com/hulryung/xcli/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/hulryung/xcli)](https://github.com/hulryung/xcli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

X (Twitter) API CLI 도구.

[English](README.md)

## 설치

### Homebrew (macOS / Linux)

```bash
brew tap hulryung/xcli
brew install xcli
```

### 바이너리 다운로드

[GitHub Releases](https://github.com/hulryung/xcli/releases/latest)에서 플랫폼별 바이너리를 다운로드할 수 있습니다.

| 플랫폼 | 파일 |
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

### 소스에서 빌드

```bash
git clone https://github.com/hulryung/xcli.git
cd xcli
cargo build --release
```

## 인증

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

## 사용법

### 트윗 작성

```bash
xcli tweet "Hello from xcli!"
# Tweet posted! ID: 1234567890
```

### 스레드 작성

긴 텍스트는 자동으로 스레드로 분할됩니다. `---` 구분자를 사용해 분할 위치를 직접 지정할 수도 있습니다.

```bash
# 긴 텍스트를 자동으로 스레드로 분할
xcli tweet "첫 번째 트윗 내용...
---
두 번째 트윗 내용...
---
세 번째 트윗 내용..."
# Thread posted! (3 tweets)
#   [1/3] ID: 1111111111
#   [2/3] ID: 2222222222
#   [3/3] ID: 3333333333

# 게시 전에 스레드 분할 미리보기
xcli tweet "긴 텍스트..." --dry-run
# Thread preview (2 tweets):
#   [1/2] (250/280) 첫 번째 청크...
#   [2/2] (180/280) 두 번째 청크...
```

### 트윗 삭제

```bash
xcli delete 1234567890
# Tweet 1234567890 deleted.
```

### 인증 관리

```bash
# OAuth 로그인 (브라우저가 열림)
xcli auth login
# Logged in as @username

# 로그인 상태 확인
xcli auth status
# Logged in as @username
# Credentials: /Users/you/.config/xcli/credentials.json

# 로그아웃 (저장된 토큰 삭제)
xcli auth logout
# Logged out. Credentials removed.
```

## 인증 우선순위

1. `~/.config/xcli/credentials.json` (OAuth login으로 저장된 토큰)
2. `.env`의 `X_ACCESS_TOKEN` / `X_ACCESS_TOKEN_SECRET`

## 라이선스

[MIT](LICENSE)
