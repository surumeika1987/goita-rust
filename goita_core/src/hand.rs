use std::collections::HashMap;

use crate::Piece;

/// A player's hand of pieces, tracking the count of each piece type.
#[derive(Debug, Clone, PartialEq)]
pub struct Hand {
    piece_counts: HashMap<Piece, u8>,
}

impl From<Vec<Piece>> for Hand {
    /// Converts a vector of [`Piece`] into a [`Hand`].
    ///
    /// This is equivalent to calling [`Hand::new_with_pieces`].
    ///
    /// # Panics
    ///
    /// Panics if the provided pieces exceed the maximum hand size (8).
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

    /// Creates a new `Hand` initialized with the given pieces.
    ///
    /// Each piece is added via [`Hand::add`], so the same validation rules apply.
    ///
    /// # Panics
    ///
    /// Panics if adding the provided pieces would exceed the maximum hand size (8).
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

    /// Adds a `Piece` to the hand.
    ///
    /// # Panics
    ///
    /// Panics if the hand already contains 8 pieces, since a hand cannot
    /// hold more than 8 pieces.
    pub fn add(&mut self, piece: Piece) {
        if self.len() >= 8 {
            panic!("Cannot add more than 8 pieces to a hand");
        }

        *self.piece_counts.entry(piece).or_insert(0) += 1;
    }

    /// Removes one instance of the specified [`Piece`] from the hand.
    ///
    /// If the piece is present with a count greater than zero, its count is
    /// decremented and internal zero-count entries are cleaned up.
    ///
    /// # Panics
    ///
    /// Panics if the specified piece is not present in the hand.
    pub fn remove(&mut self, piece: Piece) {
        match self.piece_counts.get_mut(&piece) {
            Some(c) if *c > 0 => {
                *c -= 1;
                self.clean_counts();
            }
            _ => panic!("Cannot remove piece {:?} from hand: not present", piece),
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
        let mut pieces = self
            .piece_counts
            .iter()
            .filter(|&(_, &c)| c > 0)
            .flat_map(|(&piece, &count)| std::iter::repeat_n(piece, count as usize))
            .collect::<Vec<Piece>>();
        pieces.sort();
        pieces
    }
}

impl Default for Hand {
    /// Creates an empty [`Hand`] using the default configuration.
    ///
    /// This is equivalent to calling [`Hand::new`].
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
