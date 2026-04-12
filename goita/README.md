# goita

goita is a Rust crate that provides **game flow logic and a public API** for Goita.  
Built on top of `goita_core`, it handles higher-level gameplay concerns such as round progression, game state management, and rule application.

[日本語 README](./README_ja.md)

## Features

- `GoitaGame`: game-level lifecycle management
- `GoitaRound`: round-level progression logic
- `GoitaRule`: configurable rule set
- `ApplyResult` / `DealEvent` / `RoundResult` / `GameResult`: typed events and outcomes

## License

This project is dual-licensed under **MIT License** or **Apache License Version 2.0**.  
You may choose either license.
