//! Core domain types for implementing the Japanese game **Goita**.
//!
//! This crate provides foundational, framework-agnostic data models for Goita game logic:
//! pieces, player hands, board state, turn-oriented actions, and team grouping.
//! It intentionally focuses on reusable primitives so higher-level crates can build
//! gameplay rules, simulation engines, AI, or UI layers on top.
//!
//! # What this crate includes
//!
//! - [`Piece`]: Piece kinds used in Goita, including built-in point values.
//! - [`DEFAULT_PIECES`]: Standard 32-piece distribution for a full set.
//! - [`Hand`]: A player's hand with count-based management (max 8 pieces).
//! - [`Board`]: Per-player placed piece history using [`BoardDirection`].
//! - [`PieceWithFacing`]: Face-up / face-down board representation.
//! - [`PlayerAction`]: High-level move representation (`Pass` or `Place`).
//! - [`Team`]: Team grouping for 4-player matches.
//!
//! # Design notes
//!
//! - `Hand` enforces an 8-piece capacity and panics for invalid operations
//!   (e.g. overfilling a hand or removing a missing piece).
//! - `Board` stores placements per direction and returns snapshots as `Vec` values.
//! - `PieceWithFacing` preserves hidden information while still tracking underlying pieces.
//! - `BoardDirection` supports clockwise rotation via [`BoardDirection::next`].
//!
//! # Examples
//!
//! Create and inspect a hand:
//!
//! ```rust
//! use goita_core::{Hand, Piece};
//!
//! let mut hand = Hand::new();
//! hand.add(Piece::Pawn);
//! hand.add(Piece::Gold);
//! hand.add(Piece::Pawn);
//!
//! assert_eq!(hand.len(), 3);
//! assert_eq!(hand.count(Piece::Pawn), 2);
//! assert!(hand.contains(Piece::Gold));
//! ```
//!
//! Build a hand with the `hand!` macro:
//!
//! ```rust
//! use goita_core::{hand, Piece};
//!
//! let h = hand! {
//!     Piece::King => 1,
//!     Piece::Pawn => 3,
//! };
//! assert_eq!(h.len(), 4);
//! ```
//!
//! Place pieces on the board with facing information:
//!
//! ```rust
//! use goita_core::{Board, BoardDirection, Piece, PieceWithFacing};
//!
//! let mut board = Board::new();
//! board.place_pieces(
//!     BoardDirection::North,
//!     PieceWithFacing::FaceDown(Piece::Pawn),
//!     Piece::Gold,
//! );
//!
//! let north = board.get_pieces(BoardDirection::North);
//! assert_eq!(north.len(), 2);
//! assert_eq!(north[0], PieceWithFacing::FaceDown(Piece::Pawn));
//! assert_eq!(north[1], PieceWithFacing::FaceUp(Piece::Gold));
//! ```
//!
//! Work with direction and team mapping:
//!
//! ```rust
//! use goita_core::{BoardDirection, Team};
//!
//! assert_eq!(BoardDirection::North.next(), BoardDirection::East);
//! assert_eq!(Team::from(BoardDirection::South), Team::NorthSouth);
//! assert_eq!(Team::from(BoardDirection::West), Team::EastWest);
//! ```
mod board;
mod hand;
mod piece;

pub use board::{Board, BoardDirection, PieceWithFacing, PlayerAction, Team};
pub use hand::Hand;
pub use piece::{DEFAULT_PIECES, Piece};
