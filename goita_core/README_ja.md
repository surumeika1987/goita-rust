# goita_core

goita_core は、将棋駒を使った日本のゲーム「ごいた」を実装するための**基礎ドメインモデル**を提供する Rust クレートです。  
上位レイヤー（ゲーム進行、シミュレーション、AI、UI など）から再利用できるよう、駒・手札・盤面・行動・チームといったプリミティブに集中しています。

[English README](./README.md)

## 主な機能

- `Piece` / `DEFAULT_PIECES`: 駒の種類と標準 32 枚構成
- `Hand`: 最大 8 枚の手札管理
- `Board` / `BoardDirection`: 方向付きの盤面管理
- `PieceWithFacing`: 表向き・裏向き情報を保持した駒表現
- `PlayerAction`: 手番の行動表現（Pass / Place）
- `Team`: 4 人戦のチーム分類

## ライセンス

このプロジェクトは **MIT License** または **Apache License Version 2.0** のデュアルライセンスです。  
必要に応じていずれかを選択して利用できます。
