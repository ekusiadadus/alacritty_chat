# 次のステップ

## 今後の実装課題

1. **PTY 統合の実装**

   - alacritty_terminal の適切な API を使用して実際の PTY を開く
   - シェルプロセスを起動し、入出力を適切に処理する
   - バックグラウンドスレッドで PTY からの出力を継続的に読み取る

2. **ターミナル描画の改善**

   - alacritty_terminal の Term 構造体から画面バッファを取得
   - egui 上で文字セル単位でレンダリングする実装
   - 色やスタイル（太字、斜体など）のサポート

3. **キーボード入力処理**

   - ターミナルがフォーカスされた時のキー入力ハンドリング
   - 特殊キー（矢印キー、ファンクションキーなど）の適切な処理
   - ショートカットキーの実装

4. **LLM API 連携**

   - OpenAI API との実際の通信実装
   - API 設定（キー、モデルなど）の管理
   - ストリーミング応答のサポート

5. **設定システム**

   - 設定ファイルの読み込み・保存機能
   - Alacritty の既存設定ファイルとの連携
   - UI 設定（テーマ、フォントなど）のカスタマイズ

6. **UI 改善**
   - チャット履歴のフォーマット改善
   - コードブロックやマークダウンのレンダリング
   - スクロールバックと検索機能

## 技術的検討事項

1. **alacritty_terminal の API 理解**

   - 現在のバージョンの API を詳細に調査
   - Term 構造体と Config 構造体の正確な使用方法
   - PTY 統合の詳細実装

2. **クロスプラットフォームの対応**

   - Windows、Linux、macOS での動作確認
   - WSL2 環境での実行方法の改善

3. **パフォーマンス最適化**

   - 描画処理の効率化
   - メモリ使用量の最適化
   - 並列処理の改善

4. **テスト戦略**
   - ユニットテストの追加
   - 統合テストの実装
   - UI 自動テストの検討

## リファクタリング候補

1. **モジュール構造の見直し**

   - 各機能を適切なサブモジュールに分割
   - 関心の分離をさらに徹底

2. **エラーハンドリングの改善**

   - エラー型の定義と統一
   - 適切なエラー表示とログ記録

3. **状態管理の改善**
   - 状態の集中管理とイベント駆動アーキテクチャの検討
   - スレッド間通信の最適化
