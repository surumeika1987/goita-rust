# goita

goita は、日本のゲーム「ごいた」の**ゲーム進行ロジックと公開 API**を提供する Rust クレートです。  
`goita_core` の基礎モデルを利用し、ラウンド進行・ゲーム状態・ルール適用などの高レベルな処理を扱います。

[English README](./README.md)

## 主な機能

- `GoitaGame`: ゲーム全体の進行管理
- `GoitaRound`: ラウンド単位の進行管理
- `GoitaRule`: ルール設定
- `ApplyResult` / `DealEvent` / `RoundResult` / `GameResult`: ゲーム進行イベントや結果の型

## ライセンス

このプロジェクトは **MIT License** または **Apache License Version 2.0** のデュアルライセンスです。  
必要に応じていずれかを選択して利用できます。
