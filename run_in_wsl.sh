#!/bin/bash

# WSL2環境でAlacritty Chatを実行するためのスクリプト

# 色の設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Alacritty Chat - WSL2実行スクリプト${NC}"
echo "======================================"

# WSL環境かどうかをチェック
if [[ ! -v WSL_DISTRO_NAME ]]; then
    echo -e "${YELLOW}このスクリプトはWSL環境用です。通常のLinux環境では単に 'cargo run' を使用してください。${NC}"
    exit 1
fi

# X Serverの設定
if [[ -z $DISPLAY ]]; then
    # WSL2のIPを取得
    WSL_IP=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
    
    echo -e "${YELLOW}DISPLAY環境変数が設定されていません。自動設定します。${NC}"
    export DISPLAY=$WSL_IP:0.0
    echo -e "DISPLAY=${GREEN}$DISPLAY${NC} に設定しました"
else
    echo -e "DISPLAY=${GREEN}$DISPLAY${NC} が設定されています"
fi

# X Serverが実行中かをチェック
echo "X Serverへの接続をテストしています..."
if ! xset q &>/dev/null; then
    echo -e "${RED}X Serverに接続できません。${NC}"
    echo "以下を確認してください："
    echo "1. Windows側でVcXsrv、Xming、X410などのX Serverが起動しているか"
    echo "2. Windowsファイアウォールで接続が許可されているか"
    echo "3. X Serverが'Disable access control'（アクセス制御無効）で起動されているか"
    
    echo -e "\n${YELLOW}VcXsrvの推奨設定：${NC}"
    echo "- 'Multiple windows' を選択"
    echo "- 'Start no client' を選択"
    echo "- 'Disable access control' にチェック"
    
    echo -e "\nWindows側でX Serverを起動した後、再度このスクリプトを実行してください。"
    exit 1
fi

echo -e "${GREEN}X Serverに接続できました！${NC}"

# プロジェクト名を.cargo/config.tomlから取得するか、ディレクトリ名を使用
PACKAGE_NAME=$(basename $(pwd))
echo "パッケージ '$PACKAGE_NAME' を実行します..."

# アプリケーションの実行
cargo run

# 終了コードの確認
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}アプリケーションは終了コード $EXIT_CODE で終了しました。${NC}"
    echo "詳細はエラーメッセージを確認してください。"
else
    echo -e "${GREEN}アプリケーションは正常に終了しました。${NC}"
fi 