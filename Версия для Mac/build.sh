#!/usr/bin/env bash
# Скрипт сборки «Мой день» для macOS
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> [1/3] Запуск автоматических тестов..."
cargo test --locked

echo "==> [2/3] Сборка релизного бинарника под macOS..."
cargo build --release --locked

APP_NAME="Мой день"
DIST_DIR="dist"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "==> [3/3] Формирование macOS App Bundle (${APP_DIR})..."
mkdir -p "${DIST_DIR}"
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

cp "target/release/study-planner" "${MACOS_DIR}/study-planner"
chmod +x "${MACOS_DIR}/study-planner"

if [ -f "bundle/Info.plist" ]; then
    cp "bundle/Info.plist" "${CONTENTS_DIR}/Info.plist"
fi

if [ -f "bundle/AppIcon.icns" ]; then
    cp "bundle/AppIcon.icns" "${RESOURCES_DIR}/AppIcon.icns"
fi

echo "========================================================"
echo " Сборка успешно завершена!"
echo " macOS Application Bundle: ${APP_DIR}"
echo " Чтобы запустить приложение: ./run.sh или open \"${APP_DIR}\""
echo "========================================================"
