#!/usr/bin/env bash
# Запуск приложения «Мой день» на macOS
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

APP_PATH="dist/Мой день.app"

if [ -d "$APP_PATH" ]; then
    echo "Открываем ${APP_PATH}..."
    open "$APP_PATH"
else
    echo "Приложение ещё не собрано в App Bundle. Запуск сборки..."
    ./build.sh
    echo "Открываем ${APP_PATH}..."
    open "$APP_PATH"
fi
