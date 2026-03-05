# heascale-tui

TUI-клиент для управления [Headscale](https://headscale.net/) через HTTP API.

## Возможности

- Вкладки: `Users`, `Nodes`, `PreAuth Keys`, `API Keys`
- Загрузка и просмотр сущностей
- Создание:
  - `User` (пошагово: `name` -> `display name` -> `email`)
  - `PreAuthKey`
  - `API key`
- Действия:
  - `User`: delete
  - `Node`: delete, expire
  - `PreAuthKey`: expire
  - `API key`: expire, delete
- Копирование в буфер обмена:
  - `Enter` на `PreAuthKey` копирует полный ключ
  - `Enter` на `API key` копирует полный ключ, если он был создан в текущей сессии; иначе копирует `prefix`

## Требования

- Rust stable (рекомендуется через `rustup`)
- Доступ к Headscale API

## Установка и запуск

```bash
git clone https://github.com/heascale/heascale-tui.git
cd heascale-tui
cargo run
```

Или с явной настройкой окружения:

```bash
HEADSCALE_URL="https://headscale.com" \
HEADSCALE_API_KEY="<your_api_key>" \
cargo run
```

## Конфигурация

Используются переменные окружения:

- `HEADSCALE_URL` (или `HEADSCALE_SERVER`) - базовый URL сервера
- `HEADSCALE_API_KEY` - API ключ Headscale

Пример:

```bash
export HEADSCALE_URL="https://headscale.com"
export HEADSCALE_API_KEY="hsk_xxx"
cargo run
```

## Горячие клавиши

- `Tab` / `Shift+Tab` / `←` / `→` - переключение вкладок
- `1..4` - перейти к вкладке
- `↑` / `↓` или `j` / `k` - навигация по таблице
- `r` - обновить текущую вкладку
- `c` - создать сущность в текущей вкладке
- `d` - удалить (где доступно)
- `e` - expire (где доступно)
- `Enter` - копировать ключ (`PreAuthKeys` / `ApiKeys`)
- `q` / `Esc` - выход

## Особенности Headscale API

- Полный секрет `API key` возвращается только в момент создания.
- `ListApiKeys` возвращает только `prefix` и метаданные.
- Поэтому старые API-ключи невозможно восстановить через API после создания.

## Логи

Приложение пишет логи в файл:

```text
./heascale-tui.log
```

## Сборка

```bash
cargo check
cargo build --release
```
