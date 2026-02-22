# AIOS Research Summary — Consilium Results

**Дата:** 2026-02-20
**Стадия:** Research (завершена)

## Ключевые решения по итогам консилиума

### Согласованные решения (все эксперты сходятся)

| Решение | Выбор |
|---------|-------|
| Build system | Debian live-build в Docker |
| Window Manager | labwc (Wayland stacking compositor) |
| Login manager | greetd (auto-login MVP, tuigreet multi-user) |
| Чат-клиент | Iced 0.14 (Rust) — markdown, syntax highlight встроены |
| Dock | Iced + iced_layershell 0.15 (Wayland layer-shell) |
| MCP SDK | rmcp 0.16.0 (официальный Rust SDK, 3.8M загрузок) |
| Браузер | Chromium (--ozone-platform-hint=auto для Wayland) |
| OTA | RAUC (A/B partitions, Phase 3) |
| Audio | PipeWire + WirePlumber |
| STT | whisper-rs 0.15 (локальный) + Whisper API (облако) |
| TTS | piper-rs 0.1.9 (локальный) + ElevenLabs/OpenAI (облако) |
| Embeddings | fastembed 5.11 (ONNX, all-MiniLM-L6-v2) |
| Лицензия | MIT |
| CI/CD | GitHub Actions (ubuntu-24.04) |
| ARM | Deferred to Phase 3, tiny-skia вместо wgpu |

### Разногласия (требуют решения)

#### 1. IPC: Unix sockets vs gRPC (tonic)
- **Архитектор:** Unix sockets + NDJSON (нулевые зависимости)
- **Rust-инженер:** gRPC tonic 0.14 (типизированный стриминг)
- **API-дизайнер:** Unix sockets + MessagePack

**Решение:** Unix sockets + length-prefixed JSON. Обоснование: gRPC добавляет protobuf codegen и ~4MB к бинарнику. Стриминг токенов — это просто поток JSON-сообщений по сокету. tonic — overkill для 2-3 процессов на одной машине. API-дизайнер уже написал полный набор Rust-типов для этого варианта.

#### 2. Базовый дистрибутив: Bookworm vs Trixie
- **Архитектор:** Trixie (labwc новее, PipeWire новее)
- **DevOps:** Bookworm (стабильность, labwc 0.8.3 есть)

**Решение:** Bookworm (stable). Обоснование: стабильность важнее новых фич для MVP. labwc 0.8.3 достаточен. PipeWire 0.3.65 работает. При необходимости — backports.

#### 3. Vector DB: SQLite+sqlite-vec vs Qdrant vs LanceDB
- **Rust-инженер:** SQLite + sqlite-vec (единое хранилище)
- **Архитектор:** Qdrant embedded

**Решение:** SQLite + sqlite-vec. Обоснование: единая БД для всего (факты, история, embeddings), нет отдельного процесса, минимальные зависимости. Alpha-статус sqlite-vec приемлем для прототипа.

### WebView в чате
Все эксперты сходятся: **нет production-ready решения** для embedded WebView внутри Iced.
- Phase 1: только нативные Iced-виджеты
- Phase 3: wry overlay (отдельное окно)

## Стек (финальный)

### Rust crates (проверены на crates.io)

```
iced               0.14.0   — GUI framework
iced_layershell    0.15.0   — Wayland layer-shell (dock)
rmcp               0.16.0   — MCP protocol
async-openai       0.33.0   — OpenAI/GPT API
misanthropic       0.5.1    — Claude API
ollama-rs          0.3.4    — Ollama local LLM
rusqlite           0.38     — SQLite
sqlite-vec         0.1.7    — Vector search extension
tantivy            0.25.0   — Full-text search
fastembed          5.11.0   — ONNX embeddings
whisper-rs         0.15.1   — Whisper STT
piper-rs           0.1.9    — Piper TTS
cpal               0.17.3   — Audio capture
rodio              0.22.0   — Audio playback
zbus               5.13.2   — D-Bus (system tools)
tokio              1.49     — Async runtime
serde/serde_json   1.0      — Serialization
tracing            0.1      — Logging
thiserror/anyhow   2.0/1.0  — Error handling
uuid               1.21     — UUIDs
chrono             0.4      — Timestamps
reqwest            0.12     — HTTP client
```

## Estimated ISO size: ~400-500MB compressed

## Безопасность: модель Transparent Execution
- TrustLevel tagging на всех данных
- aios-confirm — отдельный процесс
- AppArmor профиль для aios-agent
- Append-only audit log (chattr +a)
- Rate limiting (3 destructive/min)
- Btrfs snapshots перед деструктивными операциями
