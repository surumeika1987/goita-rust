use std::collections::HashMap;

use crate::Piece;

/// Represents the direction of a player's pieces on the board. Each direction corresponds to a
/// specific player in the game. This is used to track which pieces belong to which player on the
/// board, as well as their orientation (facing up or down).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardDirection {
    /// Player 1
    North,
    /// Player 2
    East,
    /// Player 3
    South,
    /// Player 4
    West,
}

impl From<BoardDirection> for usize {
    /// Converts a [`BoardDirection`] into its corresponding numeric index.
    ///
    /// The mapping is:
    /// - `North` -> `0`
    /// - `East` -> `1`
    /// - `South` -> `2`
    /// - `West` -> `3`
    fn from(d: BoardDirection) -> Self {
        match d {
            BoardDirection::North => 0,
            BoardDirection::East => 1,
            BoardDirection::South => 2,
            BoardDirection::West => 3,
        }
    }
}

impl From<u8> for BoardDirection {
    /// Converts a `u8` value into a [`BoardDirection`].
    ///
    /// The conversion is based on `value % 4`:
    /// - `0` => [`BoardDirection::North`]
    /// - `1` => [`BoardDirection::East`]
    /// - `2` => [`BoardDirection::South`]
    /// - `3` => [`BoardDirection::West`]
    ///
    /// # Panics
    /// Panics if an unexpected remainder is produced (defensive fallback case).
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => BoardDirection::North,
            1 => BoardDirection::East,
            2 => BoardDirection::South,
            3 => BoardDirection::West,
            _ => panic!("Invalid value for BoardDirection: {}", value),
        }
    }
}

impl BoardDirection {
    /// Returns the next clockwise direction.
    ///
    /// The rotation order is:
    /// `North -> East -> South -> West -> North`.
    pub fn next(self) -> Self {
        match self {
            BoardDirection::North => BoardDirection::East,
            BoardDirection::East => BoardDirection::South,
            BoardDirection::South => BoardDirection::West,
            BoardDirection::West => BoardDirection::North,
        }
    }
}

/// An action a player can take on their turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    /// Skip the turn without placing any pieces.
    Pass,
    /// Place two pieces on the board in order: `top` then `bottom`.
    Place { top: Piece, bottom: Piece },
}

/// Teams in a 4-player game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    /// The team formed by the North and South players.
    NorthSouth,
    /// The team formed by the East and West players.
    EastWest,
}

impl From<BoardDirection> for Team {
    fn from(d: BoardDirection) -> Self {
        match d {
            BoardDirection::North | BoardDirection::South => Team::NorthSouth,
            BoardDirection::East | BoardDirection::West => Team::EastWest,
        }
    }
}

/// Represents a piece on the board along with its facing (up or down). In Goita, pieces can be
/// placed face-up (visible to all players) or face-down (hidden from all players). This struct
/// encapsulates both the piece type and its orientation on the board.
// TODO: Change Up to FaceUp and Down to FaceDown for better clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceWithFacing {
    /// The piece is face-up and it content is visible to all players.
    Up(Piece),
    /// The piece is face-down and its content is hidden from all players.
    Down(Piece),
}

impl From<PieceWithFacing> for Piece {
    /// Converts a [`PieceWithFacing`] into its underlying [`Piece`],
    /// ignoring whether it is facing up or down.
    fn from(pwf: PieceWithFacing) -> Self {
        match pwf {
            PieceWithFacing::Up(p) | PieceWithFacing::Down(p) => p,
        }
    }
}

/// Represents the state of the board in a game of Goita. The board is represented as a mapping from
/// each board direction (player) to a list of pieces they have on the board, along with their
/// facing (up or down). This allows us to track the state of the board for each player, including
/// which pieces they have placed and whether those pieces are visible or hidden.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    // The pieces on the board, organized by player (board direction). Each player can have up to 8
    // pieces on the board, and each piece can be either face-up (visible) or face-down (hidden).
    // The `HashMap` allows us to easily look up the pieces for each player and manage the state of
    // the board.
    pieces: HashMap<BoardDirection, Vec<PieceWithFacing>>,
    // The last player who placed pieces on the board. This can be used to determine turn order and
    // enforce game rules related to piece placement. It is `None` if no pieces have been placed
    // yet.
    last_placed_player: Option<BoardDirection>,
}

impl Board {
    /// Creates a new, empty board with no pieces placed.
    pub fn new() -> Self {
        Self {
            pieces: HashMap::new(),
            last_placed_player: None,
        }
    }

    /// Returns a reference to the list of pieces for the given player (board direction). If the
    /// player has no pieces on the board, it returns an empty list. This method allows us to easily
    /// access the pieces for a specific player and check their state (facing) as needed.
    pub fn get_pieces(&self, direction: BoardDirection) -> Vec<PieceWithFacing> {
        self.pieces
            .get(&direction)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Returns a flat list of all pieces on the board, regardless of player or facing. This can be
    /// useful for evaluating the overall state of the board, counting pieces, or implementing game
    /// logic that needs to consider all pieces in play. Each piece in the list includes its facing
    /// (up or down) so that the caller can determine whether it is visible or hidden.
    pub fn get_all_pieces(&self) -> Vec<PieceWithFacing> {
        self.pieces
            .values()
            .flat_map(|list| list.iter().cloned())
            .collect()
    }

    /// Places two pieces on the board for the specified direction.
    ///
    /// The `top_piece` is pushed first, then `bottom_piece` is converted to
    /// an upward-facing piece and pushed second.
    ///
    /// # Panics
    ///
    /// Panics if adding these two pieces would exceed the maximum of 8 pieces
    /// allowed for the given direction.
    ///
    /// # Parameters
    ///
    /// - `direction`: The board direction where the pieces are placed.
    /// - `top_piece`: The piece to place on top.
    /// - `bottom_piece`: The piece to place on the bottom (stored as `Up` facing).
    pub fn place_pieces(
        &mut self,
        direction: BoardDirection,
        top_piece: PieceWithFacing,
        bottom_piece: Piece,
    ) {
        let list = self.pieces.entry(direction).or_default();
        if list.len() + 2 > 8 {
            panic!(
                "Cannot place pieces for {:?}: would exceed maximum of 8 pieces on the board",
                direction
            );
        }
        list.push(top_piece);
        list.push(PieceWithFacing::Up(bottom_piece));
    }
}

impl Default for Board {
    /// Returns the default `Board` value by delegating to [`Board::new`].
    fn default() -> Self {
        Self::new()
    }
}
