# LINE Admin App

RustとTauriで構築されたLINE公式アカウント管理デスクトップアプリケーション

## 機能

### 1. LINE公式アカウント連携
- **Messaging API + Webhook設定**: LINEユーザーからのメッセージを受信
- **自動DB登録**: 受信したユーザー情報とメッセージをSQLiteデータベースに保存
- **通知機能**: LINE NotifyとSlackへの通知送信

### 2. データ管理画面
- **管理者用Web UI**: ユーザー情報、メッセージ履歴の閲覧・管理
- **ユーザー管理**: LINE友だち追加したユーザーの一覧表示
- **メッセージ履歴**: 受信したメッセージの確認

### 3. スケジュール配信機能
- **予約配信**: 指定した日時にメッセージを自動送信
- **ブロードキャスト**: 全ユーザーへの一斉配信
- **個別配信**: 特定ユーザーへの配信

## 技術スタック

- **フレームワーク**: Tauri 2.1
- **バックエンド**: Rust
  - Webサーバー: Axum
  - データベース: SQLite (sqlx)
  - スケジューラー: tokio-cron-scheduler
  - HTTP Client: reqwest
- **フロントエンド**: HTML/CSS/JavaScript

## セットアップ

### 前提条件

- Rust (最新の安定版)
- Node.js (Tauriのビルドに必要)

### インストール

1. リポジトリをクローン
```bash
git clone <repository-url>
cd line_admin_app
```

2. 依存関係をインストール
```bash
cargo build
```

3. 環境変数を設定
```bash
cp .env.example .env
# .env ファイルを編集して必要な設定を追加
```

### 実行

開発モードで実行:
```bash
cargo tauri dev
```

本番用ビルド:
```bash
cargo tauri build
```

### デスクトップアプリのリリース（macOS/Windows）

#### GitHub Actionsで自動ビルド（推奨）

1. GitHubにリポジトリをプッシュ
2. バージョンタグを作成してプッシュ:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. GitHub ActionsがmacOS版（Intel/Apple Silicon）とWindows版を自動ビルド
4. Releasesページからダウンロード可能

生成されるファイル:
- `LINE Admin App_0.1.0_aarch64.dmg` (macOS Apple Silicon)
- `LINE Admin App_0.1.0_x64.dmg` (macOS Intel)
- `LINE Admin App_0.1.0_x64-setup.exe` (Windows)

#### ローカルでビルド

**macOS**:
```bash
cargo tauri build --target aarch64-apple-darwin  # Apple Silicon
cargo tauri build --target x86_64-apple-darwin   # Intel
```

**Windows** (Windows環境で実行):
```bash
cargo tauri build --target x86_64-pc-windows-msvc
```

## 設定

### LINE Developersの設定

1. [LINE Developers Console](https://developers.line.biz/)でチャネルを作成
2. Messaging API設定から以下を取得:
   - Channel Access Token
   - Channel Secret
3. Webhook URLを設定:
   ```
   http://your-server:3000/webhook/line
   ```
   ※ 本番環境ではHTTPS必須（ngrokなどを使用）

### アプリケーション設定

アプリケーションの「設定」タブから以下を設定:

- **LINE Channel Access Token**: Messaging APIのアクセストークン
- **LINE Channel Secret**: Webhook署名検証用シークレット
- **LINE Notify Token**: 通知送信用トークン（オプション）
- **Slack Webhook URL**: Slack通知用URL（オプション）

## 使い方

### 1. ユーザー管理
- 「ユーザー管理」タブでLINE友だち追加したユーザーを確認
- ユーザーID、表示名、登録日時を表示

### 2. メッセージ履歴
- 「メッセージ履歴」タブで受信したメッセージを確認
- テキスト、画像、動画、スタンプなど様々な形式に対応

### 3. スケジュール配信
- 「スケジュール配信」タブから新規配信を作成
- 配信先、メッセージ内容、配信日時を指定
- 登録済みスケジュールの一覧確認

### 4. 通知
- ユーザーからメッセージを受信すると自動的に通知
- LINE NotifyまたはSlackに送信（設定済みの場合）

## データベース構造

- **users**: LINEユーザー情報
- **messages**: 受信メッセージ
- **scheduled_messages**: スケジュール配信
- **calendars**: カレンダーイベント
- **settings**: アプリケーション設定
- **notification_logs**: 通知ログ

## Webhook エンドポイント

- `GET /`: ヘルスチェック
- `POST /webhook/line`: LINE Messaging API Webhook

## トラブルシューティング

### Webhookが動作しない
- LINE Developers ConsoleでWebhook URLが正しく設定されているか確認
- サーバーが起動しており、外部からアクセス可能か確認
- 本番環境ではHTTPSが必須

### 通知が送信されない
- 設定タブでLINE Notify TokenまたはSlack Webhook URLが正しく設定されているか確認
- トークン/URLが有効か確認

### データベースエラー
- データベースファイルのパスが正しいか確認
- 書き込み権限があるか確認

## ライセンス

MIT License

Copyright (c) 2025 石津恭輔 (Kyosuke Ishizu)

## 開発者

石津恭輔 (Kyosuke Ishizu)

Built with Tauri and Rust
