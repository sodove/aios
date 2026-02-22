# E2E Scenario: AIOS MVP
Платформы: Docker (ISO build smoke-test)

## Шаги
- [x] 1. `cargo build --workspace` компилируется без ошибок (проверено)
- [x] 2. `cargo clippy --workspace` без ошибок, 0 warnings (проверено)
- [x] 3. `cargo test --workspace` 5 тестов passed, 0 failed (проверено)
- [x] 4. `make build-linux` — Docker cross-compilation всех 4 бинарников для Linux x86_64 (проверено)
- [x] 5. `make install-binaries` — бинарники скопированы в iso/config/includes.chroot/usr/local/bin/ (проверено)
- [x] 6. `make build-iso` — ISO собран через live-build в Docker (проверено, 974MB)
- [x] 7. Проверить что ISO файл создан в output/ и имеет разумный размер (>200MB) — 974MB (проверено)

## QEMU Boot Smoke Test
- [x] 8. ISO загружается в QEMU (SeaBIOS → ISOLINUX → Linux 6.1.0-43-amd64) (проверено)
- [x] 9. Все systemd сервисы стартуют (dbus, NetworkManager, greetd, logind, wpa_supplicant) (проверено)
- [x] 10. Сеть поднимается (e1000: ens3 NIC Link is Up 1000 Mbps) (проверено)
- [x] 11. Логин `aios:aios` работает → shell `aios@debian:~$` (проверено)
- [x] 12. `which aios-agent aios-chat aios-dock aios-confirm` — все 4 бинарника в PATH (проверено)
- [x] 13. `ls -lh /usr/local/bin/aios-*` — agent 11M, chat 26M, confirm 22M, dock 22M (проверено)
- [x] 14. greetd config: `command = "sway"`, `user = "aios"` (проверено)
- [x] 15. sway config: AIOS-кастомный с keybinds и autostart (проверено)
- [x] 16. Ядро: Linux debian 6.1.0-43-amd64 x86_64 (проверено)
