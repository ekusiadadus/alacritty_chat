# Alacritty Chat

Alacritty と egui ベースのチャット GUI パネルを組み合わせたアプリケーションです。Alacritty 本体を変更せずに、ターミナルとチャットパネルを左右に分割表示します。

## 機能

- egui ベースの GUI でシンプルなレイアウト
- 左側にチャットパネル、右側に Alacritty ターミナル
- チャットパネルでの LLM API 通信（モック実装）
- パネルのリサイズが可能

## プロジェクト構成

```
src/
├── main.rs      # アプリケーションのエントリーポイント
├── app.rs       # アプリケーションの状態管理とレイアウト
├── terminal.rs  # ターミナルパネルの実装
└── chat.rs      # チャットパネルとLLM通信の実装
```

## 実装方針

このプロジェクトでは、以下の方針で開発を進めています：

1. **Alacritty のターミナルエンジン活用**: `alacritty_terminal`クレートを依存として使用し、Alacritty のコードを改変せずに利用
2. **egui による GUI 実装**: シンプルで軽量な egui フレームワークで GUI を構築
3. **サイドパネルレイアウト**: 左右に分割して、チャットとターミナルを並列表示
4. **LLM API 通信**: 非同期処理で API リクエストを行い、UI スレッドをブロックしない設計

## 開発ステータス

現在は基本的な UI レイアウトと仮実装が完了しています。今後、以下の機能を実装予定です：

- 実際の PTY（擬似端末）を使用したシェル統合
- ターミナル画面の描画（行・列の文字セル、カラー対応）
- OpenAI API（または他の LLM API）との実際の通信
- キーボード入力のハンドリング改善
- スクロールバックと選択機能

## 必要な環境

- Rust 2021 以降
- X11 または Wayland 環境

## ビルドと実行

```sh
# ビルド
cargo build

# 実行
cargo run
```

## WSL2 での実行

WSL2 環境で GUI アプリケーションを実行するには、X Server の設定が必要です。以下の方法のいずれかを使用してください：

### 方法 1: 付属のスクリプトを使用（推奨）

プロジェクトに含まれる実行スクリプトを使用すると、必要な設定を自動的に行います：

```sh
# スクリプトに実行権限を付与
chmod +x run_in_wsl.sh

# スクリプトを実行
./run_in_wsl.sh
```

このスクリプトは以下を行います：

- 適切な DISPLAY 環境変数の設定
- X Server への接続テスト
- エラー時の詳細な診断情報の表示

### 方法 2: 手動設定

1. Windows 側で X Server をインストール・起動：

   - [VcXsrv](https://sourceforge.net/projects/vcxsrv/)（推奨）
   - [Xming](https://sourceforge.net/projects/xming/)
   - [X410](https://x410.dev/)（有料、最も高機能）

2. X Server を以下の設定で起動：

   - Multiple windows モード
   - Display number: 0
   - Start no client
   - **重要**: 「Disable access control」にチェック

3. WSL2 側で DISPLAY 環境変数を設定：

   ```sh
   # WSL2 の IP アドレスを取得して設定
   export DISPLAY=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}'):0.0
   ```

4. アプリケーションを実行：
   ```sh
   cargo run
   ```

### トラブルシューティング

- **接続エラー「Broken pipe」**:

  - X Server が起動しているか確認
  - Windows Defender などのファイアウォールで接続が許可されているか確認
  - 「Disable access control」オプションが有効になっているか確認

- **起動はするが描画されない**:

  - DISPLAY の設定値が正しいか確認（WSL2 の IP は再起動ごとに変わる可能性があります）
  - 環境変数を設定し直してみてください

- **WSLg (Windows 11) を使用している場合**:
  - WSLg が有効であれば、X Server を起動せずに直接実行できます
  - それでも問題がある場合は上記の従来の方法を試してください

## 参考リソース

- [alacritty_terminal API ドキュメント](https://docs.rs/alacritty_terminal/)
- [egui ドキュメント](https://docs.rs/egui/)
- [System76 COSMIC Terminal](https://github.com/pop-os/cosmic-term) - alacritty_terminal を利用した実装例
- [WSL2 での GUI アプリ実行ガイド](https://docs.microsoft.com/ja-jp/windows/wsl/tutorials/gui-apps)
