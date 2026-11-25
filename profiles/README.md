# PS5169 register profiles

[Русская версия](README.ru.md)

The [TB371FC profile](tb371fc.toml) contains the register sequences taken from the original Lenovo driver. USB 3 and DisplayPort have been tested on TB371FC. I cannot assume that the same initialization, equalization, and switching sequences apply to every board with a PS5169. Each board can select a profile validated with its own hardware via `PS5169_PROFILE`. The TB371FC board project keeps a copy of this profile with its USB wiring.
