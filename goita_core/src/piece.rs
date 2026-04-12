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
    /// Knight (馬)
    Knight,
    /// Lance (香)
    Lance,
    /// Pawn (し)
    Pawn,
}

impl Piece {
    /// Goita pieces have different point values based on their type. This method returns the point
    /// Value of the piece, which can be used for scoring and strategic evaluation in the game. The
    /// point values are assigned as follows:
    /// King: 50 points
    /// Rook: 40 points
    /// Bishop: 40 points
    /// Gold: 30 points
    /// Silver: 30 points
    /// Knight: 20 points
    /// Lance: 20 points
    /// Pawn: 10 points
    pub fn point_value(&self) -> u8 {
        match self {
            Piece::King => 50,
            Piece::Rook => 40,
            Piece::Bishop => 40,
            Piece::Gold => 30,
            Piece::Silver => 30,
            Piece::Knight => 20,
            Piece::Lance => 20,
            Piece::Pawn => 10,
        }
    }
}

/// Default piece distribution used to initialize a standard Goita set.
///
/// Each tuple represents `(Piece, count)`, and the total number of pieces is 32:
/// - King: 2
/// - Rook: 2
/// - Bishop: 2
/// - Gold: 4
/// - Silver: 4
/// - Knight: 4
/// - Lance: 4
/// - Pawn: 10
pub const DEFAULT_PIECES: [(Piece, u8); 8] = [
    (Piece::King, 2),
    (Piece::Rook, 2),
    (Piece::Bishop, 2),
    (Piece::Gold, 4),
    (Piece::Silver, 4),
    (Piece::Knight, 4),
    (Piece::Lance, 4),
    (Piece::Pawn, 10),
];
