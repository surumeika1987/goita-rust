use crate::{ApplyResult, DealEvent, Error, HandRank, types::InvalidPlaceError};
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
    last_placed_player: Option<BoardDirection>,
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
            last_placed_player: None,
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

        // 必ずpanicしないことが保証されているため、unwrapしても安全
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

        for (key, original_count) in original_count_map.iter() {
            let count = count_map.get(key).unwrap_or(&0);

            if original_count != count {
                return Err(Error::InvalidHand);
            }
        }

        // この関数の最初ですでにhandの形状を検査しているため、unwrapしても安全
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

    /// 配牌時の役（DealEvent）を判定する。
    ///
    /// 判定順は「歩8枚 → 歩7枚 → 歩6枚」。
    /// - 歩8枚: `EightPawn`
    /// - 歩7枚: 歩以外の1枚の点数を2倍して `SevenPawn`
    /// - 歩6枚: 残り2枚が同種ならその点数を2倍、異種なら高い方の点数で `SixPawn`
    ///
    /// いずれにも該当しない場合は、通常の配牌結果（役なし）を返す。
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
            // 必ず１枚の駒が残るためunwrapしても安全
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
            // 必ず2枚の駒が残るためunwrapしても安全
            let first_piece = remain_pieces.pop().unwrap();
            let second_piece = remain_pieces.pop().unwrap();

            let double_up = first_piece == second_piece;

            return DealEvent::HandRank {
                player: *player,
                rank: HandRank::SixPawn {
                    score: if double_up {
                        first_piece.point_value() * 2
                    } else if first_piece.point_value() > second_piece.point_value() {
                        first_piece.point_value()
                    } else {
                        second_piece.point_value()
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

    /// Returns a vector of the pieces currently placed on the board for the specified player.
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
                    if let Some(last_placed_player) = self.last_placed_player {
                        if player == last_placed_player {
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

    // プレイヤーが2枚の駒（上段・下段）を場に配置する。
    //
    // 処理内容:
    // - 指定した2枚が手札に存在するかを検証する
    //   - 同一駒2枚なら2枚所持が必要
    //   - 別駒なら両方を1枚以上所持している必要がある
    // - 上段駒の向きを決定する
    //   - 同一プレイヤーの連続配置、または初回配置は伏せ駒
    //   - それ以外は表駒
    // - 上段が表駒の場合、直前の場札との整合性を検証する
    //   - 王は特殊制約（直前が王・香・歩なら不可）
    //   - 王以外は直前の駒と同種である必要がある
    // - 下段が王の場合、王配置可能条件 (`can_place_king`) を満たしているか確認する
    // - 盤面へ配置し、手札から2枚を除去し、最終配置プレイヤーを更新する
    //
    // # Errors
    // - `Error::PieceNotInHand`: 必要な駒が手札にない場合
    // - `Error::InvalidPlace(InvalidPlaceError::PieceMismatch)`: 表駒の一致条件に違反した場合
    // - `Error::InvalidPlace(InvalidPlaceError::InvalidKingPlacement)`: 王配置条件を満たさない場合
    fn place_pieces(
        &mut self,
        player: BoardDirection,
        top_piece: Piece,
        bottom_piece: Piece,
    ) -> Result<(), Error> {
        let hand = &self.hands[player as usize];

        if top_piece == bottom_piece {
            if hand.count(top_piece) < 2 {
                return Err(Error::PieceNotInHand);
            }
        } else if hand.count(top_piece) == 0 || hand.count(bottom_piece) == 0 {
            return Err(Error::PieceNotInHand);
        }

        let top_piece_with_face = if let Some(last_placed_player) = self.last_placed_player {
            if player == last_placed_player {
                PieceWithFacing::FaceDowmn(top_piece)
            } else {
                PieceWithFacing::FaceUp(top_piece)
            }
        } else {
            PieceWithFacing::FaceDowmn(top_piece)
        };

        if let Err(error) = self.check_place_pieces(player, top_piece_with_face, bottom_piece) {
            return Err(Error::InvalidPlace(error));
        }

        // 手札は8枚なので8枚以上置くことはないため、必ずpanicしないことが保証されている
        self.board
            .place_pieces(player, top_piece_with_face, bottom_piece);
        // 関数の最初で手札の形状を検査しているため、必ず手札にtop_pieceとbottom_pieceが存在することが保証されている
        self.hands[player as usize].remove(top_piece);
        self.hands[player as usize].remove(bottom_piece);

        self.last_placed_player = Some(player);

        Ok(())
    }

    /// Validates whether a player can place the given pair of pieces in the current round state.
    ///
    /// Rules:
    /// - If the top piece is face-up:
    ///   - A previous placed piece must exist; otherwise placement is rejected.
    ///   - The face-up piece must match the last placed piece, except for the special King rule.
    ///   - If the face-up piece is `King`, it is rejected when the last placed piece is
    ///     `King`, `Lance`, or `Pawn`; otherwise it is allowed.
    /// - If the top piece is face-down:
    ///   - The acting player must be the same as the last player who placed.
    /// - If the bottom piece is `King`:
    ///   - Placement is allowed only when `can_place_king(player)` returns true.
    ///
    /// Returns `Ok(())` when placement is valid; otherwise returns `InvalidPlaceError`.
    pub fn check_place_pieces(
        &self,
        player: BoardDirection,
        top_piece_with_face: PieceWithFacing,
        bottom_piece: Piece,
    ) -> Result<(), InvalidPlaceError> {
        match top_piece_with_face {
            PieceWithFacing::FaceUp(piece) => {
                let Some(last_placed_piece) = self.last_placed_piece() else {
                    return Err(InvalidPlaceError::FaceUpNotAllowed);
                };

                if piece == Piece::King {
                    match last_placed_piece {
                        Piece::King | Piece::Lance | Piece::Pawn => {
                            return Err(InvalidPlaceError::PieceMismatch {
                                expected: last_placed_piece,
                                actual: piece,
                            });
                        }
                        _ => {}
                    }
                } else if piece != last_placed_piece {
                    return Err(InvalidPlaceError::PieceMismatch {
                        expected: last_placed_piece,
                        actual: piece,
                    });
                }
            }
            PieceWithFacing::FaceDowmn(_) => {
                if player != self.last_placed_player.unwrap_or(player) {
                    return Err(InvalidPlaceError::FaceUpNotAllowed);
                }
            }
        }

        if bottom_piece == Piece::King && !self.can_place_king(player) {
            return Err(InvalidPlaceError::InvalidKingPlacement);
        }

        Ok(())
    }

    // 次のターンに移行する。現在のターンプレイヤーを更新する。
    fn next_turn(&mut self) {
        self.current_turn_player = self.current_turn_player.next();
    }

    // ラウンド終了条件を判定し、成立していれば結果を返す。
    //
    // 判定内容:
    // - 直前に駒を置いたプレイヤーが存在すること
    // - そのプレイヤーの場札が8枚そろっていること
    //
    // 得点計算:
    // - 下段の駒の点数を基本得点とする
    // - ただし、上段が「同じ駒の伏せ駒（ダブル）」なら得点を2倍にする
    //
    // 条件を満たした場合は `RoundResult` を返し、未成立なら `None` を返す。
    fn check_round_over(&self) -> Option<crate::RoundResult> {
        if let Some(last_placed_player) = self.last_placed_player {
            let pieces = self.board.get_pieces(last_placed_player);
            if pieces.len() == 8 {
                let top_piece = pieces[6];
                let bottom_piece = Piece::from(pieces[7]);

                let double_up =
                    matches!(top_piece, PieceWithFacing::FaceDowmn(piece) if piece == bottom_piece);

                let score = if double_up {
                    bottom_piece.point_value() * 2
                } else {
                    bottom_piece.point_value()
                };

                return Some(crate::RoundResult::new(last_placed_player, score));
            }
        }

        None
    }

    /// Returns the most recently placed piece on the board.
    ///
    /// This checks `last_placed_player` and then retrieves the last piece
    /// from that player's placed pieces on the board.
    ///
    /// # Returns
    /// - `Some(Piece)` if a last placed piece exists.
    /// - `None` if no player has placed a piece yet, or no piece is found.
    pub fn last_placed_piece(&self) -> Option<Piece> {
        if let Some(last_placed_player) = self.last_placed_player
            && let Some(piece) = self.board.get_pieces(last_placed_player).last()
        {
            return Some(Piece::from(*piece));
        }

        None
    }

    // 王を出せる条件:
    // 1. 自分の手札に王が2枚ある
    // 2. 自分の場札がすでに6枚ある
    // 3. 場全体で表向きの王がちょうど1枚ある
    // いずれかを満たせば配置可能
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
            .filter(|p| **p == PieceWithFacing::FaceUp(Piece::King))
            .count()
            == 1
        {
            return true;
        }

        false
    }

    /// Returns the direction of the player who most recently placed a piece.
    ///
    /// If no piece has been placed yet in the current round, this returns `None`.
    pub fn last_placed_player(&self) -> Option<BoardDirection> {
        return self.last_placed_player;
    }
}
