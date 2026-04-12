use std::collections::HashMap;

use crate::Piece;

/// A player's hand of pieces, tracking the count of each piece type.
#[derive(Debug, Clone, PartialEq)]
pub struct Hand {
    piece_counts: HashMap<Piece, u8>,
}

impl From<Vec<Piece>> for Hand {
    fn from(pieces: Vec<Piece>) -> Self {
        Self::new_with_pieces(pieces)
    }
}

impl Hand {
    /// Creates an empty hand.
    pub fn new() -> Self {
        Self {
            piece_counts: HashMap::new(),
        }
    }

    /// Creates a hand initialized with the given pieces. The input vector can contain duplicates,
    pub fn new_with_pieces(pieces: Vec<Piece>) -> Self {
        let mut hand = Self::new();
        for piece in pieces {
            hand.add(piece);
        }
        hand
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
    /// use goita_core::{Hand, Piece};
    ///
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

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructs a `Hand` from repeated piece entries.
///
/// This macro accepts one or more `piece => count` pairs and expands them
/// into a vector by pushing each `piece` value `count` times, in declaration order.
/// The resulting vector is then converted into a `Hand` via `Hand::from`.
///
/// # Arguments
/// - `piece`: A piece expression to insert.
/// - `count`: The number of times to insert that piece.
///
/// # Returns
/// A `Hand` containing all expanded pieces.
///
/// # Example
/// ```rust
/// use goita_core::{hand, Piece};
///
/// let h = hand! {
///     Piece::Pawn => 3,
///     Piece::King => 1,
/// };
/// assert_eq!(h.len(), 4);
/// ```
#[macro_export]
macro_rules! hand {
    ($($piece:expr => $count:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $(
            for _ in 0..$count {
                v.push($piece);
            }
        )*
        $crate::Hand::from(v)
    }};
}
