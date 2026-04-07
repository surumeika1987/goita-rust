use std::collections::HashMap;

/// Represents a shogi piece used in Goita (ごいた).
///
/// Goita is a traditional board game from the Noto Peninsula (能登半島) of Japan,
/// played with a subset of shogi pieces. Each variant corresponds to a specific
/// piece with its own role in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    /// King (王)
    King,
    /// Rook (飛)
    Rook,
    /// Bishop (角)
    Bishop,
    /// Gold General (金)
    Gold,
    /// Silver General (銀)
    Silver,
    /// Knight (桂)
    Knight,
    /// Lance (香)
    Lance,
    /// Pawn (し)
    Pawn,
}

/// A player's hand of pieces, tracking the count of each piece type.
#[derive(Debug, Clone, PartialEq)]
pub struct Hand {
    piece_counts: HashMap<Piece, u8>
}

impl Hand {
    /// Creates an empty hand.
    pub fn new() -> Self {
        Self { piece_counts: HashMap::new() }
    }

    /// Returns the total number of pieces in the hand.
    pub fn len(&self) -> u8 {
        self.piece_counts.values().sum()
    }

    /// Returns `true` if the hand is empty.
    pub fn is_empty(&self) -> bool {
        self.piece_counts.values().all(|&c| c == 0)
    }

    /// Adds a piece to the hand. Returns `true` if the piece was added, or `false` if the hand
    /// already has 8 pieces.
    pub fn add(&mut self, piece: Piece) -> bool {
        if self.len() >= 8 {
            return false;
        }

        *self.piece_counts.entry(piece).or_insert(0) += 1;
        true
    }

    /// Removes one of the given piece from the hand.
    /// Returns `true` if a piece was removed.
    pub fn remove(&mut self, piece: Piece) -> bool {
        match self.piece_counts.get_mut(&piece) {
            Some(c) if *c > 0 => {
                *c -= 1;
                self.clean_counts();
                true
            }
            _ => false,
        }
    }

    // Removes all pieces with zero count from the hand. This is useful for keeping the internal
    // state clean after removals.
    fn clean_counts(&mut self) {
        self.piece_counts.retain(|_, &mut c| c > 0);
    }

    /// Returns the number of the given piece in the hand.
    pub fn count(&self, piece: Piece) -> u8 {
        *self.piece_counts.get(&piece).unwrap_or(&0)
    }

    /// Returns `true` if the hand contains at least one of the given piece.
    pub fn contains(&self, piece: Piece) -> bool {
        self.count(piece) > 0
    }

    /// Returns an iterator over `(piece, count)` pairs with non-zero counts.
    pub fn iter(&self) -> impl Iterator<Item = (&Piece, &u8)> {
        self.piece_counts.iter().filter(|&(_, &c)| c > 0)
    }

    /// Returns all pieces in the hand as a flat list.
    /// Pieces with a count greater than 1 appear multiple times.
    ///
    /// # Example
    /// ```
    /// let mut hand = Hand::new();
    /// hand.add(Piece::Pawn);
    /// hand.add(Piece::Pawn);
    /// hand.add(Piece::Gold);
    /// assert_eq!(hand.pieces().len(), 3);
    /// ```
    pub fn pieces(&self) -> Vec<Piece> {
        self.piece_counts
            .iter()
            .filter(|&(_, &c)| c > 0)
            .flat_map(|(&piece, &count)| std::iter::repeat(piece).take(count as usize))
            .collect()
    }
}

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

/// Represents a piece on the board along with its facing (up or down). In Goita, pieces can be
/// placed face-up (visible to all players) or face-down (hidden from all players). This struct
/// encapsulates both the piece type and its orientation on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceWithFacing {
    /// The piece is face-up and it content is visible to all players.
    Up(Piece),
    /// The piece is face-down and its content is hidden from all players.
    Down(Piece),
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
        Self { pieces: HashMap::new(), last_placed_player: None }
    }

    /// Returns a reference to the list of pieces for the given board direction (player), or `None`
    /// if the player has not placed any pieces on the board. Each piece in the list includes its
    /// facing (up or down), allowing us to determine which pieces are visible and which are hidden
    /// for that player.
    pub fn get_pieces(&self, direction: BoardDirection) -> Option<&[PieceWithFacing]> {
        self.pieces.get(&direction).map(|vec| vec.as_slice())
    }
    
    /// Places two pieces on the board for a given player (direction). Each player can place up to 8
    /// pieces on the board. This method returns `true` if the pieces were successfully placed, or
    /// `false` if the player already has 8 pieces on the board and cannot place more.
    pub fn place_pieces(&mut self, direction: BoardDirection, top_piece: PieceWithFacing, bottom_piece: PieceWithFacing) -> bool{
        let list = self.pieces
            .entry(direction)
            .or_insert_with(Vec::new);
        if list.len() >= 8 {
            return false;
        }
        list.push(top_piece);
        list.push(bottom_piece);
        true
    }
}
