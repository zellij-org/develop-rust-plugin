## About
![img-2024-11-07-172647](https://github.com/user-attachments/assets/d55af6de-f4f7-4f30-9aa0-eb27574b9f0c)

This is helper plugin for developing other Zellij plugins. It's intended to be run in a folder with an existing Zellij plugin, assisting in its development by shortening the feedback loop and mental load involved in compiling and running the plugin.

By default it binds the `Ctrl Shift r` key (see below for rebinding) to:
1. Run `cargo build`
2. Start or reload the plugin

More about Zellij plugins: [Zellij Documentation][docs]
An example Zellij plugin (good to use to get started): [Rust Plugin Example][example]

[zellij]: https://github.com/zellij-org/zellij
[docs]: https://zellij.dev/documentation/plugins.html
[example]: https://github.com/zellij-org/rust-plugin-example

## How to run
Open the `Plugin Manager` (by default: `Ctrl o` + `p`), press `Ctrl a`, paste the following url and press `Enter`: 

https://github.com/zellij-org/develop-rust-plugin/releases/latest/download/develop-rust-plugin.wasm

## Configuration
It's possible to change the `reload_shortcut` (by default `Ctrl Shift r`) to any other shortcut by specifying it in the `reload_shortcut` plugin configuration. eg.

```kdl
reload_shortcut "Ctrl a"
```
