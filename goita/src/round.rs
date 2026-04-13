use crate::{ApplyResult, DealEvent, Error, HandRank};
use goita_core::{
    Board, BoardDirection, DEFAULT_PIECES, Hand, Piece, PieceWithFacing, PlayerAction, Team,
};
use rand::prelude::*;
use std::collections::HashMap;

/// Represents the state of a single Goita round.
///
/// This struct tracks:
/// - the current board state,
/// - whose turn it is,
/// - who last placed a piece (if any),
/// - each of the four players' hands, and
/// - whether the round has ended.
#[derive(Debug)]
pub struct GoitaRound {
    /// The current board state.
    board: Board,
    /// The player whose turn it is.
    current_turn_player: BoardDirection,
    /// The player who most recently placed a piece, or `None` if no piece has been placed yet.
    last_place_player: Option<BoardDirection>,
    /// The hands of all four players.
    hands: [Hand; 4],
    /// Indicates whether this round is over.
    round_is_over: bool,
}

impl GoitaRound {
    /// Creates a new `GoitaRound` with the specified starting player.
    ///
    /// # Arguments
    /// * `init_turn_player` - The player who takes the first turn in this round.
    ///
    /// # Returns
    /// A newly initialized `GoitaRound` with:
    /// - an empty board,
    /// - no last placed player,
    /// - empty hands for all four players, and
    /// - `round_is_over` set to `false`.
    pub fn new(init_turn_player: BoardDirection) -> Self {
        Self {
            board: Board::new(),
            current_turn_player: init_turn_player,
            last_place_player: None,
            hands: [Hand::new(), Hand::new(), Hand::new(), Hand::new()],
            round_is_over: false,
        }
    }

    /// Shuffles the full piece set and deals hands to all four players.
    ///
    /// This method expands `DEFAULT_PIECES` into a full deck, shuffles it with the
    /// provided random number generator, splits it into 4 hands of 8 pieces, and
    /// delegates validation/state update to `deal_hands`.
    ///
    /// # Arguments
    /// * `rng` - Random number generator used to shuffle the expanded deck.
    ///
    /// # Returns
    /// Returns the resulting `DealEvent` after dealing and validating hands.
    pub fn shuffle_and_deal_hands(&mut self, rng: &mut rand::rngs::StdRng) -> DealEvent {
        let mut expanded: Vec<Piece> = DEFAULT_PIECES
            .iter()
            .flat_map(|(piece, count)| std::iter::repeat_n(*piece, *count as usize))
            .collect();

        expanded.shuffle(rng);

        let hands = expanded
            .chunks(8)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<Piece>>>();

        self.deal_hands(hands).unwrap()
    }

    /// Deals and validates all players' hands.
    ///
    /// This function requires exactly four hands with eight pieces each.
    /// It verifies that the aggregated piece counts across all hands match
    /// `DEFAULT_PIECES` exactly.
    ///
    /// If validation succeeds, `self.hands` is updated and a deal event is checked.
    /// When the deal event is `DealEvent::FivePawnSameTeam` or `DealEvent::HandRank`,
    /// `self.round_is_over` is set to `true`.
    ///
    /// # Arguments
    /// * `hands` - A vector containing four player hands, each with eight `Piece`s.
    ///
    /// # Returns
    /// * `Ok(DealEvent)` - The detected deal event after updating the hands.
    /// * `Err(Error::InvalidHand)` - Returned when hand shape or piece counts are invalid.
    pub fn deal_hands(&mut self, hands: Vec<Vec<Piece>>) -> Result<DealEvent, Error> {
        if hands.len() != 4 || hands.iter().any(|hand| hand.len() != 8) {
            return Err(Error::InvalidHand);
        }

        let original_count_map: HashMap<Piece, u8> =
            DEFAULT_PIECES
                .iter()
                .fold(HashMap::new(), |mut map, (piece, count)| {
                    *map.entry(*piece).or_insert(0) += *count;
                    map
                });
        let count_map: HashMap<Piece, u8> =
            hands
                .iter()
                .flatten()
                .fold(HashMap::new(), |mut map, piece| {
                    *map.entry(*piece).or_insert(0) += 1;
                    map
                });

        for key in original_count_map.keys() {
            let original_count = original_count_map.get(key).unwrap();
            let count = count_map.get(key).unwrap_or(&0);

            if original_count != count {
                return Err(Error::InvalidHand);
            }
        }

        self.hands = hands
            .iter()
            .map(|hand| Hand::from(hand.clone()))
            .collect::<Vec<Hand>>()
            .try_into()
            .unwrap();

        let deal_event = self.check_deal_event();

        match deal_event {
            DealEvent::FivePawnSameTeam { team: _ }
            | DealEvent::HandRank { rank: _, player: _ } => self.round_is_over = true,
            _ => {}
        };

        Ok(deal_event)
    }

    fn check_deal_event(&self) -> DealEvent {
        const DIRECTIONS: [BoardDirection; 4] = [
            BoardDirection::North,
            BoardDirection::East,
            BoardDirection::South,
            BoardDirection::West,
        ];

        let pawn_count = self
            .hands
            .iter()
            .map(|hand| hand.count(Piece::Pawn))
            .enumerate()
            .map(|(i, count)| (DIRECTIONS[i], count))
            .collect::<Vec<(BoardDirection, u8)>>();

        let event_player = pawn_count.iter().find(|(_, count)| *count == 8);
        if let Some((player, _)) = event_player {
            return DealEvent::HandRank {
                player: *player,
                rank: HandRank::EightPawn,
            };
        }

        let event_player = pawn_count.iter().find(|(_, count)| *count == 7);
        if let Some((player, _)) = event_player {
            let pieces = self.hands[*player as usize].pieces();
            let remain_piece = pieces.iter().find(|p| **p != Piece::Pawn).unwrap();
            return DealEvent::HandRank {
                player: *player,
                rank: HandRank::SevenPawn {
                    score: remain_piece.point_value() * 2,
                },
            };
        }

        let event_player = pawn_count.iter().find(|(_, count)| *count == 6);
        if let Some((player, _)) = event_player {
            let pieces = self.hands[*player as usize].pieces();
            let mut remain_pieces = pieces
                .iter()
                .filter(|p| **p != Piece::Pawn)
                .cloned()
                .collect::<Vec<Piece>>();
            let first_piece = remain_pieces.pop().unwrap();
            let secound_piece = remain_pieces.pop().unwrap();

            let double_up = first_piece == secound_piece;

            return DealEvent::HandRank {
                player: *player,
                rank: HandRank::SixPawn {
                    score: if double_up {
                        first_piece.point_value() * 2
                    } else if first_piece.point_value() > secound_piece.point_value() {
                        first_piece.point_value()
                    } else {
                        secound_piece.point_value()
                    },
                },
            };
        }

        let event_players = pawn_count
            .iter()
            .filter(|(_, count)| *count >= 5)
            .map(|(dir, _)| *dir)
            .collect::<Vec<BoardDirection>>();
        if event_players.len() == 2 {
            let team1 = Team::from(event_players[0]);
            let team2 = Team::from(event_players[1]);
            if team1 == team2 {
                return DealEvent::FivePawnSameTeam { team: team1 };
            } else {
                return DealEvent::FivePawnBothTeams;
            }
        } else if event_players.len() == 1 {
            return DealEvent::FivePawn {
                player: event_players[0],
            };
        }

        DealEvent::Normal
    }

    /// Returns a shared reference to the specified player's hand.
    pub fn player_hand(&self, player: BoardDirection) -> &Hand {
        &self.hands[player as usize]
    }

    pub fn player_board(&self, player: BoardDirection) -> Vec<PieceWithFacing> {
        self.board.get_pieces(player)
    }

    /// Returns the player whose turn it currently is.
    pub fn current_turn_player(&self) -> Option<BoardDirection> {
        if self.round_is_over {
            None
        } else {
            Some(self.current_turn_player)
        }
    }

    /// Returns whether the current round has ended.
    pub fn round_is_over(&self) -> bool {
        self.round_is_over
    }

    /// Applies a player's action and returns whether the round continues or ends.
    pub fn action(
        &mut self,
        player: BoardDirection,
        action: PlayerAction,
    ) -> Result<ApplyResult, Error> {
        if let Some(turn_player) = self.current_turn_player() {
            if player != turn_player {
                return Err(Error::NotYourTurn);
            }
            match action {
                PlayerAction::Pass => {
                    if let Some(last_place_player) = self.last_place_player {
                        if player == last_place_player {
                            return Err(Error::InvalidPass);
                        }
                    } else {
                        return Err(Error::InvalidPass);
                    }
                }
                PlayerAction::Place { top, bottom } => {
                    self.place_pieces(player, top, bottom)?;
                }
            }

            let round_result = self
                .check_round_over()
                .map(ApplyResult::RoundOver)
                .unwrap_or(ApplyResult::Continuing);
            if let ApplyResult::RoundOver(_) = round_result {
                self.round_is_over = true;
            }
            self.next_turn();
            Ok(round_result)
        } else {
            Err(Error::RoundIsOver)
        }
    }

    fn place_pieces(
        &mut self,
        player: BoardDirection,
        top_piece: Piece,
        bottom_piece: Piece,
    ) -> Result<(), Error> {
        let player_index = player as usize;
        let hand = &self.hands[player_index];

        if top_piece == bottom_piece {
            if hand.count(top_piece) < 2 {
                return Err(Error::PieceNotInHand);
            }
        } else if hand.count(top_piece) == 0 || hand.count(bottom_piece) == 0 {
            return Err(Error::PieceNotInHand);
        }

        let top_piece_with_face = if let Some(last_place_player) = self.last_place_player {
            if player == last_place_player {
                PieceWithFacing::Down(top_piece)
            } else {
                PieceWithFacing::Up(top_piece)
            }
        } else {
            PieceWithFacing::Down(top_piece)
        };

        if let PieceWithFacing::Up(piece) = top_piece_with_face {
            let Some(last_placed_piece) = self.get_last_placed_piece() else {
                return Err(Error::InvalidPlace);
            };

            if piece == Piece::King {
                match last_placed_piece {
                    Piece::King | Piece::Lance | Piece::Pawn => {
                        return Err(Error::InvalidPlace);
                    }
                    _ => {}
                }
            } else if piece != last_placed_piece {
                return Err(Error::InvalidPlace);
            }
        }

        if bottom_piece == Piece::King && !self.can_place_king(player) {
            return Err(Error::InvalidPlace);
        }

        if !self
            .board
            .place_pieces(player, top_piece_with_face, bottom_piece)
        {
            return Err(Error::InvalidPlace);
        }

        self.hands[player_index].remove(top_piece);
        self.hands[player_index].remove(bottom_piece);

        self.last_place_player = Some(player);

        Ok(())
    }

    fn next_turn(&mut self) {
        self.current_turn_player = self.current_turn_player.next();
    }

    fn check_round_over(&self) -> Option<crate::RoundResult> {
        if let Some(last_place_player) = self.last_place_player {
            let pieces = self.board.get_pieces(last_place_player);
            if pieces.len() == 8 {
                let top_piece = pieces[6];
                let bottom_piece = Piece::from(pieces[7]);

                let double_up =
                    matches!(top_piece, PieceWithFacing::Down(piece) if piece == bottom_piece);

                let score = if double_up {
                    bottom_piece.point_value() * 2
                } else {
                    bottom_piece.point_value()
                };

                return Some(crate::RoundResult::new(last_place_player, score));
            }
        }

        None
    }

    fn get_last_placed_piece(&self) -> Option<Piece> {
        if let Some(last_place_player) = self.last_place_player
            && let Some(piece) = self.board.get_pieces(last_place_player).last()
        {
            return Some(Piece::from(*piece));
        }

        None
    }

    fn can_place_king(&self, player: BoardDirection) -> bool {
        if self.hands[player as usize].count(Piece::King) == 2 {
            return true;
        }
        if self.board.get_pieces(player).len() == 6 {
            return true;
        }
        if self
            .board
            .get_all_pieces()
            .iter()
            .filter(|p| **p == PieceWithFacing::Up(Piece::King))
            .count()
            == 1
        {
            return true;
        }

        false
    }
}
