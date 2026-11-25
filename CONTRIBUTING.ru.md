# Как участвовать в разработке PM8150B-SMB5-driver

[English version](CONTRIBUTING.md)

Соблюдайте [соглашение о качестве кода](CODE_QUALITY.ru.md) и пишите сообщения коммитов по [Conventional Commits](COMMIT_CONVENTIONS.ru.md) на английском. Подключение и настройки конкретной платы храните в проекте этой платы.

Перед отправкой pull request запустите проверки из корня рабочего пространства.

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Опишите, что изменилось и как вы это проверили. Для изменений, затрагивающих работу с оборудованием, укажите окружение Linux и оборудование, если проверяли работу на устройстве.
