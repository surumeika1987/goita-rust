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
pub struct Hand {
    counts: HashMap<Piece, u8>
}

impl Hand {
    /// Creates an empty hand.
    pub fn new() -> Self {
        Self { counts: HashMap::new() }
    }

    /// Returns the total number of pieces in the hand.
    pub fn len(&self) -> u8 {
        self.counts.values().map(|&c| c as usize).sum()
    }

    /// Returns `true` if the hand is empty.
    pub fn is_empty(&self) -> bool {
        self.counts.values().all(|&c| c == 0)
    }

    /// Adds a piece to the hand.
    pub fn add(&mut self, piece: Piece) {
        *self.counts.entry(piece).or_insert(0) += 1;
    }

    /// Removes one of the given piece from the hand.
    /// Returns `true` if a piece was removed.
    pub fn remove(&mut self, piece: Piece) -> bool {
        match self.counts.get_mut(&piece) {
            Some(c) if *c > 0 => { *c -= 1; true }
            _ => false,
        }
    }

    /// Returns the number of the given piece in the hand.
    pub fn count(&self, piece: Piece) -> u8 {
        *self.counts.get(&piece).unwrap_or(&0)
    }

    /// Returns `true` if the hand contains at least one of the given piece.
    pub fn contains(&self, piece: Piece) -> bool {
        self.count(piece) > 0
    }

     /// Returns an iterator over `(piece, count)` pairs with non-zero counts.
    pub fn iter(&self) -> impl Iterator<Item = (&Piece, &u8)> {
        self.counts.iter().filter(|(_, &c)| c > 0)
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
        self.counts
            .iter()
            .filter(|(_, &c)| c > 0)
            .flat_map(|(&piece, &count)| std::iter::repeat(piece).take(count as usize))
            .collect()
    }
}

pub struct Table {
}
