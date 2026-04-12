# goita_core

goita_core is a Rust crate that provides **core domain models** for implementing the Japanese game Goita.  
It focuses on reusable primitives such as pieces, hands, board state, actions, and teams, so higher-level layers (gameplay rules, simulation, AI, UI) can build on top.

[日本語 README](./README_ja.md)

## Features

- `Piece` / `DEFAULT_PIECES`: piece kinds and the standard 32-piece set
- `Hand`: hand management with an 8-piece capacity
- `Board` / `BoardDirection`: directional board state
- `PieceWithFacing`: up/down facing representation
- `PlayerAction`: turn actions (`Pass` / `Place`)
- `Team`: team grouping for 4-player games

## License

This project is dual-licensed under **MIT License** or **Apache License Version 2.0**.  
You may choose either license.
