//! Core game logic and public API for Goita.
//!
//! This crate provides the `GoitaGame` and `GoitaRound` types, game rules,
//! action results, and related domain types for building Goita applications.
//!
//! # Example
//! ```rust
//! use goita::{BoardDirection, GoitaGame, GoitaRule};
//!
//! let mut game = GoitaGame::new_with_seed(GoitaRule::default(), BoardDirection::North, 42);
//! let _deal_event = game.start_new_round()?;
//!
//! assert!(game.check_game_over().is_none());
//! # Ok::<(), goita::Error>(())
//! ```

mod game;
mod round;
mod types;

pub use game::GoitaGame;
pub use goita_core::{BoardDirection, Piece, PieceWithFacing, PlayerAction, Team};
pub use round::GoitaRound;
pub use types::{
    ApplyResult, DealEvent, Error, GameResult, GoitaRule, HandRank, InvalidPlaceError, RoundResult,
};
