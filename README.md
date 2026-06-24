# たぶん作業中

Discordの「プレイ中」表示に使う名前を、プリセットまたは手入力で切り替えるWindows向けTauriアプリです。

## 機能

- プリセットから表示名を選択
- 任意の表示名を保存・削除
- 選択した名前のステータス用プロセスを起動
- ステータス切り替え後も操作ウィンドウを維持
- 現在の表示名をウィンドウタイトルへ同期
- 再起動後に前回の表示名を復元
- ステータス用プロセスの停止

## ダウンロード

[Releases](https://github.com/EraWeb3333/tabun-working/releases) から最新版をダウンロードしてください。

- `たぶん作業中_セットアップ_v1.0.0_x64.exe`
  - 推奨
  - Windows 10 / 11（64ビット）
  - WebView2オフラインインストーラー同梱
  - 管理者権限なしで現在のユーザーへインストール
- `たぶん作業中.exe`
  - インストール不要のポータブル版
  - WebView2が導入済みのWindows向け
  - ファイル名は変更しないでください

コード署名は行っていないため、Windows SmartScreenの警告が表示される場合があります。

## Discordでの使い方

1. アプリを起動します。
2. プリセットまたは手入力で表示名を決めます。
3. `この名前で起動` を押します。
4. Discordの `ユーザー設定` → `登録済みのゲーム` を開きます。
5. 起動中の表示名をゲームとして追加します。

新しい表示名を初めて使用する場合、その名称のプロセスをDiscordへ一度追加する必要があります。

## 開発

必要環境:

- Windows 10 / 11 x64
- Node.js
- Rust MSVC toolchain
- Visual Studio C++ Build Tools

```powershell
npm install
npm run tauri:dev
```

通常のリリースビルド:

```powershell
npm run tauri:build -- --no-bundle
```

WebView2オフラインランタイムを含むNSISインストーラー:

```powershell
npm run tauri:build
```

生成先:

```text
src-tauri/target/release/tabun_working.exe
src-tauri/target/release/bundle/nsis/
```

## 技術構成

- Tauri 2
- Rust
- TypeScript
- Vite

## ライセンス

[MIT License](LICENSE)

