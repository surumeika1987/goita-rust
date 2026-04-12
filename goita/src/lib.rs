pub use goita_core::*;
use rand;
use rand::prelude::*;

/// Errors that can occur while processing a game action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// The GameNotStarted.
    GameNotStarted,
    /// The action was attempted by a player who is not the current turn player.
    NotYourTurn,
    /// The specified piece is not available in the player's hand.
    PieceNotInHand,
    /// The piece placement is not valid under the game rules.
    // TODO: Add reason details (e.g. "top piece cannot be placed face-up", "king cannot be placed
    // on bottom if certain conditions are not met", etc.)
    InvalidPlace,
    /// The pass action is not valid in the current game state.
    InvalidPass,
    /// The round has already ended, so no further actions are allowed.
    RoundIsOver,
    /// The game has already ended, so no further actions and start new rounds are allowed.
    GameIsOver,
}

/// Represents a special hand rank that is checked immediately after dealing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandRank {
    /// Six pawns in hand. Holds the score (`u8`) determined by the remaining tiles.
    SixPawn { score: u8 },
    /// Seven pawns in hand. Holds the score (`u8`) determined by the remaining tiles.
    SevenPawn { score: u8 },
    /// Eight pawns in hand. Always scores 100 points.
    EightPawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DealEvent {
    FivePawn {
        player: BoardDirection,
    },
    FivePawnSameTeam {
        team: Team,
    },
    /// Two players from different teams both have five or more Shi in hand.
    FivePawnBothTeams,
    HandRank {
        player: BoardDirection,
        rank: HandRank,
    },
    Normal,
}

/// Result of a completed round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoundResult {
    /// Last player who placed a piece in the round, which is relevant for determining the winning
    /// team and score.
    winning_player: BoardDirection,
    /// Points earned for this round.
    score: u8,
}

impl RoundResult {
    /// Creates a new `RoundResult` with the given winning team and score.
    pub fn new(winning_player: BoardDirection, score: u8) -> Self {
        Self {
            winning_player,
            score,
        }
    }

    /// Returns the team that won the round.
    pub fn winning_team(&self) -> Team {
        Team::from(self.winning_player)
    }

    /// Returns the player who placed the winning pieces in the round.
    pub fn winning_player(&self) -> BoardDirection {
        self.winning_player
    }

    /// Returns the score awarded for the round.
    pub fn score(&self) -> u8 {
        self.score
    }
}

/// Represents the outcome of a game, including the winning team and both teams' scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameResult {
    /// The team that won the game.
    winning_team: Team,
    /// The final score for the North-South team.
    north_south_score: u32,
    /// The final score for the East-West team.
    east_west_score: u32,
}

impl GameResult {
    /// Creates a new `GameResult` with the winning team and final scores for both teams.
    pub fn new(winning_team: Team, north_south_score: u32, east_west_score: u32) -> Self {
        Self {
            winning_team,
            north_south_score,
            east_west_score,
        }
    }

    /// Returns the team that won the game.
    pub fn winning_team(&self) -> &Team {
        &self.winning_team
    }

    /// Returns the final score of the North-South team.
    pub fn north_south_score(&self) -> u32 {
        self.north_south_score
    }

    /// Returns the final score of the East-West team.
    pub fn east_west_score(&self) -> u32 {
        self.east_west_score
    }

    /// Returns the score for the specified `team`.
    pub fn score(&self, team: Team) -> u32 {
        match team {
            Team::NorthSouth => self.north_south_score,
            Team::EastWest => self.east_west_score,
        }
    }
}

/// Defines the game rule configuration for Goita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoitaRule {
    /// The score required to win the game.
    winning_score: u32,
}

impl Default for GoitaRule {
    /// Creates the default rule with a winning score of `150`.
    fn default() -> Self {
        Self::new(150)
    }
}

impl GoitaRule {
    /// Creates a new `GoitaRule` with the specified winning score.
    pub fn new(winning_score: u32) -> Self {
        Self { winning_score }
    }

    /// Returns the score required to win the game.
    pub fn winning_score(&self) -> u32 {
        self.winning_score
    }
}

/// Represents the outcome of applying a game action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplyResult {
    /// The round should continue with no winner yet.
    Continuing,
    /// The round has ended with a final result.
    RoundOver(RoundResult),
}

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

    /// Shuffles the default 32-piece set and deals hands to all four players.
    ///
    /// This method:
    /// - expands `DEFAULT_PIECES` into a full deck of 32 pieces,
    /// - shuffles the deck using the provided RNG,
    /// - deals 8 pieces to each player as `Hand`,
    /// - stores the 4 resulting hands in `self.hands`,
    /// - evaluates the initial deal event via `check_deal_event`.
    ///
    /// If the deal event is either `FivePawnSameTeam` or `HandRank`,
    /// `self.round_is_over` is set to `true`.
    ///
    /// # Parameters
    /// - `rng`: Random number generator used for shuffling.
    ///
    /// # Returns
    /// The `DealEvent` detected immediately after dealing.
    ///
    /// # Panics
    /// Panics if the expanded deck cannot be converted into exactly four
    /// 8-piece hands (the implementation assumes a valid 32-piece deck).
    // 駒の既定構成を展開してシャッフルし、8枚ずつ4人分の手札として配る。
    // - `DEFAULT_PIECES` の (駒種, 枚数) から全32枚の配列を生成
    // - 乱数で順序をランダム化
    // - 8枚ごとに `Hand` を作成し、4人分の `self.hands` に格納
    // - 32枚ちょうどで4分割できる前提のため `try_into().unwrap()` を使用
    pub fn shuffle_and_deal_hands(&mut self, rng: &mut rand::rngs::StdRng) -> DealEvent {
        let mut expanded: Vec<Piece> = DEFAULT_PIECES
            .iter()
            .flat_map(|(piece, count)| std::iter::repeat(*piece).take(*count as usize))
            .collect();

        expanded.shuffle(rng);

        self.hands = expanded
            .chunks(8)
            .map(|chunk| Hand::from(chunk.to_vec()))
            .collect::<Vec<Hand>>()
            .try_into()
            .unwrap();

        let deal_event = self.check_deal_event();

        match deal_event {
            DealEvent::FivePawnSameTeam { team: _ }
            | DealEvent::HandRank { rank: _, player: _ } => self.round_is_over = true,
            _ => {}
        };

        deal_event
    }

    // 配牌直後の役イベントを判定する。
    //
    // 判定優先度:
    // 1. 8し（歩8枚）
    // 2. 7し（歩7枚 + 残り1枚の点数×2）
    // 3. 6し（歩6枚 + 残り2枚。同種ならその点数×2、異種なら高い方）
    // 4. 5し（歩5枚以上）
    //    - 2人発生で同一チーム: FivePawnSameTeam
    //    - 2人発生で別チーム: FivePawnBothTeams
    //    - 1人のみ発生: FivePawn
    // 5. 上記なし: Normal
    //
    // 各プレイヤーの歩枚数を方角付きで集計し、上から順に最初に成立したイベントを返す。
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

        // 8し判定
        let event_player = pawn_count.iter().find(|(_, count)| *count == 8);
        if let Some((player, _)) = event_player {
            return DealEvent::HandRank {
                player: *player,
                rank: HandRank::EightPawn,
            };
        }

        // 7し判定
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

        // 6し判定
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
                    } else {
                        if first_piece.point_value() > secound_piece.point_value() {
                            first_piece.point_value()
                        } else {
                            secound_piece.point_value()
                        }
                    },
                },
            };
        }

        // 5し判定
        let event_players = pawn_count
            .iter()
            .filter(|(_, count)| *count >= 5)
            .map(|(dir, _)| *dir)
            .collect::<Vec<BoardDirection>>();
        if event_players.len() == 2 {
            // 2人いる場合
            let team1 = Team::from(event_players[0]);
            let team2 = Team::from(event_players[1]);
            if team1 == team2 {
                return DealEvent::FivePawnSameTeam { team: team1 };
            } else {
                return DealEvent::FivePawnBothTeams;
            }
        } else if event_players.len() == 1 {
            // 1人だけいる場合
            return DealEvent::FivePawn {
                player: event_players[0],
            };
        }

        DealEvent::Normal
    }

    /// Returns a shared reference to the specified player's hand.
    ///
    /// # Parameters
    /// - `player`: The board direction identifying the target player.
    ///
    /// # Returns
    /// A shared reference to that player's `Hand`.
    pub fn player_hand(&self, player: BoardDirection) -> &Hand {
        &self.hands[player as usize]
    }

    /// Returns the player whose turn it currently is.
    ///
    /// # Returns
    /// - `Some(BoardDirection)` if the round is still in progress.
    /// - `None` if the round has already ended.
    pub fn current_turn_player(&self) -> Option<BoardDirection> {
        if self.round_is_over {
            None
        } else {
            Some(self.current_turn_player)
        }
    }

    /// Returns whether the current round has ended.
    ///
    /// # Returns
    /// `true` if the round is over, otherwise `false`.
    pub fn round_is_over(&self) -> bool {
        self.round_is_over
    }

    /// Applies a player's action and returns whether the round continues or ends.
    ///
    /// Validation and behavior:
    /// - Returns `Error::NotYourTurn` if `player` is not the current turn player.
    /// - Returns `Error::RoundIsOver` if the round has already ended.
    /// - `PlayerAction::Pass` is valid only if another player made the last placement;
    ///   otherwise returns `Error::InvalidPass`.
    /// - `PlayerAction::Place { top, bottom }` delegates piece placement to `place_pieces`.
    ///
    /// After applying the action, this method evaluates round completion via
    /// `check_round_over`:
    /// - returns `ApplyResult::RoundOver(...)` when finished and sets `round_is_over = true`,
    /// - otherwise returns `ApplyResult::Continuing`.
    ///
    /// The turn is then advanced with `next_turn()`.
    ///
    /// # Errors
    /// - `Error::NotYourTurn`
    /// - `Error::RoundIsOver`
    /// - `Error::InvalidPass`
    /// - Any error propagated from `place_pieces`
    // プレイヤーの行動を適用し、結果（継続 or ラウンド終了）を返す。
    // - 手番プレイヤーでない場合は `NotYourTurn`
    // - ラウンド終了済みなら `RoundIsOver`
    // - `Pass` は「直前に他プレイヤーが配置している場合のみ」有効
    // - `Place` は `place_pieces` で配置処理を実行
    // - 行動後に `check_round_over` で終了判定し、終了時は `round_is_over` を true にする
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

    // 指定プレイヤーが top/bottom の2枚を場に出せるかを検証し、問題なければ配置して手札を更新する。
    // - 手札所持チェック:
    //   - 同一駒2枚出しは2枚以上所持している必要がある
    //   - 異なる駒2枚出しは両方を1枚以上所持している必要がある
    // - top 駒の向き決定:
    //   - 直前に置いたプレイヤーと同一なら Down、異なるなら Up
    //   - 初回配置は Down
    // - top が Up の場合の合法手チェック:
    //   - 直前の場札が存在しないなら不正
    //   - top が王なら、直前が 香/歩 のときは不正
    //   - top が王以外なら、直前の駒と同種でなければ不正
    // - bottom が王の場合は can_place_king による追加制約を満たす必要がある
    // - board.place_pieces が失敗した場合は不正
    // - 成功時は手札から2枚を削除し、最終配置プレイヤーを更新する
    // - 手番を次のプレイヤーに進める
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

        let _ = self.hands[player_index].remove(top_piece);
        let _ = self.hands[player_index].remove(bottom_piece);

        self.last_place_player = Some(player);

        Ok(())
    }

    // 手番を次のプレイヤーへ進める。
    fn next_turn(&mut self) {
        self.current_turn_player = self.current_turn_player.next();
    }

    // ラウンド終了条件を判定し、成立していれば勝利チームと得点を返す。
    // - 最後に駒を置いたプレイヤーが存在し、その場札が8枚のときに終了判定を行う
    // - 7枚目(top)が裏向きかつ8枚目(bottom)と同種なら「ダブル上がり」として得点2倍
    // - 勝利チームは最後に置いたプレイヤーの方角（南北/東西）で決定
    // - 得点は bottom の点数（ダブル上がり時は2倍）
    // - 条件を満たさない場合は None を返す
    fn check_round_over(&self) -> Option<RoundResult> {
        if let Some(last_place_player) = self.last_place_player {
            let pieces = self.board.get_pieces(last_place_player);
            if pieces.len() == 8 {
                let top_piece = pieces[6];
                let bottom_piece = Piece::from(pieces[7]);

                let double_up = match top_piece {
                    PieceWithFacing::Down(piece) if piece == bottom_piece => true,
                    _ => false,
                };

                let score = if double_up {
                    bottom_piece.point_value() * 2
                } else {
                    bottom_piece.point_value()
                };

                return Some(RoundResult::new(last_place_player, score));
            }
        }

        None
    }

    // 最後に駒を置いたプレイヤーの場札の末尾を参照し、
    // その駒（表/裏の向きに関わらない本体の Piece）を返す。
    // 最終配置プレイヤーや場札が存在しない場合は None を返す。
    fn get_last_placed_piece(&self) -> Option<Piece> {
        if let Some(last_place_player) = self.last_place_player {
            if let Some(piece) = self.board.get_pieces(last_place_player).last() {
                return Some(Piece::from(*piece));
            }
        }

        None
    }

    // 王を下段に置けるかを判定する。
    // - 自分の手札に王が2枚ある場合は配置可能
    // - 盤面全体で「表向きの王」がちょうど1枚ある場合も配置可能
    // - 8枚目として王を置くときはいつでも配置可能
    // 上記以外は配置不可。
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
            .collect::<Vec<_>>()
            .len()
            == 1
        {
            return true;
        }

        false
    }
}

/// Represents the overall game state across rounds.
///
/// Stores the cumulative team scores and the currently active round.
#[derive(Debug)]
pub struct GoitaGame {
    // game rule configuration, which can be extended in the future to include more parameters if
    // needed.
    game_rule: GoitaRule,
    // Cumulative score for the North-South team.
    ns_score: u32,
    // Cumulative score for the East-West team.
    ew_score: u32,
    // Round state currently in progress.
    current_round: Option<GoitaRound>,
    // Starting player for the next round, which is determined by the previous round's result.
    round_start_player: BoardDirection,
    // Random number generator for shuffling, initialized with a seed that can be set for
    // reproducibility.
    rng: rand::rngs::StdRng,
}

impl Default for GoitaGame {
    /// Creates the default configuration: a first-to-150-point rule,
    /// with North as the starting player in the first round.
    fn default() -> Self {
        Self::new(GoitaRule::default(), BoardDirection::North)
    }
}

impl GoitaGame {
    /// Creates a new game state with zeroed team scores and an initialized first round.
    ///
    /// param `initial_round_start_player` The player who will start the first round.
    pub fn new(game_rule: GoitaRule, initial_round_start_player: BoardDirection) -> Self {
        let mut rng = rand::rng();
        let seed: u64 = rng.random();
        Self::new_with_seed(game_rule, initial_round_start_player, seed)
    }

    /// Creates a new game state with a deterministic random number generator.
    ///
    /// This constructor initializes scores to `0`, sets `current_round` to `None`,
    /// and uses `seed` to initialize the internal RNG for reproducible behavior.
    ///
    /// # Parameters
    /// - `game_rule`: The rules used for the game.
    /// - `initial_round_start_player`: The player who starts the first round.
    /// - `seed`: The seed used to initialize the RNG.
    ///
    /// # Returns
    /// A newly initialized `Self` instance.
    pub fn new_with_seed(
        game_rule: GoitaRule,
        initial_round_start_player: BoardDirection,
        seed: u64,
    ) -> Self {
        Self {
            game_rule,
            ns_score: 0,
            ew_score: 0,
            current_round: None,
            round_start_player: initial_round_start_player,
            rng: rand::SeedableRng::seed_from_u64(seed),
        }
    }

    /// Starts a new round if the game is not over.
    ///
    /// This method checks whether the game has already ended. If so, it returns
    /// `Error::GameIsOver`. Otherwise, it creates a new `GoitaRound` with
    /// `round_start_player`, shuffles and deals hands, and stores it in
    /// `self.current_round`.
    ///
    /// # Returns
    ///
    /// - `Ok(DealEvent)` containing the result of the initial hand check after dealing.
    /// - `Err(Error::GameIsOver)` if the game has already ended.
    pub fn start_new_round(&mut self) -> Result<DealEvent, Error> {
        if let Some(_) = self.check_game_over() {
            return Err(Error::GameIsOver);
        }
        let mut round = GoitaRound::new(self.round_start_player);
        let deal_event = round.shuffle_and_deal_hands(&mut self.rng);
        match deal_event {
            DealEvent::FivePawnSameTeam { team } => match team {
                Team::NorthSouth => self.ns_score = self.game_rule.winning_score(),
                Team::EastWest => self.ew_score = self.game_rule.winning_score(),
            },
            DealEvent::HandRank { player, rank } => {
                let winning_team = Team::from(player);
                let score = match rank {
                    HandRank::SixPawn { score } | HandRank::SevenPawn { score } => score as u32,
                    HandRank::EightPawn => 100,
                };
                match winning_team {
                    Team::NorthSouth => self.ns_score += score,
                    Team::EastWest => self.ew_score += score,
                }
            }
            _ => {}
        }
        self.current_round = Some(round);
        Ok(deal_event)
    }

    /// Applies a player's action for the current turn.
    ///
    /// This method validates game state, forwards the action to the active round,
    /// and updates match-level state when a round ends.
    ///
    /// When the round result is `RoundOver`, it:
    /// - adds the round score to the winning team,
    /// - checks whether the winning score has been reached and returns
    ///   `ApplyResult::GameOver` if so,
    /// - otherwise updates `round_start_player` to the winning team's first player.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - `Error::GameNotStarted` if there is no active round,
    /// - `Error::NotYourTurn` if the action is attempted by a player who is not the current turn
    /// player,
    /// - `Error::PieceNotInHand` if the player tries to place a piece they do not have,
    /// - `Error::InvalidPlace` if the piece placement violates game rules,
    /// - `Error::InvalidPass` if the pass action is not valid in the current game state,
    /// - `Error::RoundIsOver` if the round has already ended.
    /// - `Error::GameIsOver` if the game has already finished,
    pub fn play_turn(
        &mut self,
        player: BoardDirection,
        action: PlayerAction,
    ) -> Result<ApplyResult, Error> {
        if let Some(_) = self.check_game_over() {
            return Err(Error::GameIsOver);
        }
        if let Some(round) = &mut self.current_round {
            let result = round.action(player, action)?;
            if let ApplyResult::RoundOver(round_result) = result {
                match round_result.winning_team() {
                    Team::NorthSouth => self.ns_score += round_result.score() as u32,
                    Team::EastWest => self.ew_score += round_result.score() as u32,
                }

                self.round_start_player = round_result.winning_player();
            }

            Ok(result)
        } else {
            Err(Error::GameNotStarted)
        }
    }

    pub fn current_turn_plyer(&self) -> Option<BoardDirection> {
        if let Some(_) = self.check_game_over() {
            return None;
        }

        self.current_round
            .as_ref()
            .and_then(|round| round.current_turn_player())
    }

    /// Returns the current score for the North-South team.
    pub fn ns_score(&self) -> u32 {
        self.ns_score
    }

    /// Returns the current score for the East-West team.
    pub fn ew_score(&self) -> u32 {
        self.ew_score
    }

    /// Returns the current score for the specified team.
    ///
    /// # Parameters
    ///
    /// - `team`: The team whose score is requested.
    ///
    /// # Returns
    ///
    /// The score associated with `team`.
    pub fn score(&self, team: Team) -> u32 {
        match team {
            Team::NorthSouth => self.ns_score,
            Team::EastWest => self.ew_score,
        }
    }

    /// Checks whether the game has reached its end condition.
    ///
    /// The game is considered over when either team's score is greater than or equal to
    /// the winning score defined by the current game rule.
    ///
    /// Returns `Some(GameResult)` with the winning team and both final scores when the game
    /// is over; otherwise returns `None`.
    pub fn check_game_over(&self) -> Option<GameResult> {
        if self.ns_score >= self.game_rule.winning_score()
            || self.ew_score >= self.game_rule.winning_score()
        {
            let winning_team = if self.ns_score >= self.ew_score {
                Team::NorthSouth
            } else {
                Team::EastWest
            };
            Some(GameResult::new(winning_team, self.ns_score, self.ew_score))
        } else {
            None
        }
    }

    /// Returns the hand of the specified player in the current round.
    ///
    /// If no round is currently active, this returns `None`.
    /// Otherwise, it returns `Some(&Hand)` for the given `player`.
    pub fn player_hand(&self, player: BoardDirection) -> Option<&Hand> {
        self.current_round
            .as_ref()
            .map(|round| round.player_hand(player))
    }

    // TODO: return ref instead of cloning the pieces if performance becomes an issue
    pub fn player_board(&self, player: BoardDirection) -> Option<Vec<PieceWithFacing>> {
        self.current_round
            .as_ref()
            .map(|round| round.board.get_pieces(player))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 通常時のゲームの流れを通してテストする。（5し等のイベントなし)
    // テスト項目
    // - DealEvent::Normal の発生と、配牌内容がシード値に基づいて固定されていることの確認
    // - ラウンド終了後にアクションを起こそうとしたときのGameNotStarted エラー
    // - 自分のターンでないときの NotYourTurn エラー
    // - パスできない場面でのInvalidPassエラー
    // - 手持ちにない駒を置こうとしたときの PieceNotInHand エラー
    // - 置くことができない駒を置こうとしたときのPlaceInvalidエラー
    // - あがり時の倍付け処理
    // - あがり時の得点が正しく計算されていること確認
    // - ラウンド終了後にアクションを起こそうとしたときのRoundIsOver エラー
    // - ゲーム終了時にアクションを起こそうとしたときのGameIsOver エラー
    // - ゲーム終了時のスコアと勝利チームの確認
    #[test]
    fn test_normal_game_flow() {
        let mut game = GoitaGame::new_with_seed(
            GoitaRule { winning_score: 50 },
            BoardDirection::North,
            12345,
        );

        // Cehck GameNotStarted error when trying to play a turn before starting a round
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "North tries to place Pawn-Pawn before starting round: {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::GameNotStarted));

        // Start the first round with normal deal (no special hand rank or 5-shi event)
        let deal_event = game.start_new_round().unwrap();
        println!("Deal event: {:?}", deal_event);
        assert_eq!(deal_event, DealEvent::Normal);

        // Assert the initial hands are as expected based on the fixed seed
        println!(
            "Hands(North): {:?}",
            game.player_hand(BoardDirection::North)
        );
        assert_eq!(
            *game.player_hand(BoardDirection::North).unwrap(),
            hand! {
                Piece::Pawn => 2,
                Piece::Lance => 3,
                Piece::Silver => 1,
                Piece::Gold => 2,
            }
        );

        println!("Hands(East): {:?}", game.player_hand(BoardDirection::East));
        assert_eq!(
            *game.player_hand(BoardDirection::East).unwrap(),
            hand! {
                Piece::Lance  => 1,
                Piece::Rook   => 1,
                Piece::Silver => 1,
                Piece::Pawn   => 1,
                Piece::Bishop => 1,
                Piece::Knight => 3,
            }
        );

        println!(
            "Hands(South): {:?}",
            game.player_hand(BoardDirection::South)
        );
        assert_eq!(
            *game.player_hand(BoardDirection::South).unwrap(),
            hand! {
                Piece::King   => 1,
                Piece::Rook   => 1,
                Piece::Pawn   => 3,
                Piece::Knight => 1,
                Piece::Silver => 1,
                Piece::Gold   => 1,
            }
        );

        println!("Hands(West): {:?}", game.player_hand(BoardDirection::West));
        assert_eq!(
            *game.player_hand(BoardDirection::West).unwrap(),
            hand! {
                Piece::King   => 1,
                Piece::Bishop => 1,
                Piece::Pawn   => 4,
                Piece::Silver => 1,
                Piece::Gold   => 1,
            }
        );

        // Check NotYourTurn error
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "West tries to place Pawn-Pawn on first turn: {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::NotYourTurn));

        // Check InvalidPass error (cannot pass on first turn)
        let action_result = game.play_turn(BoardDirection::North, PlayerAction::Pass);
        println!("North tries to pass on first turn: {:?}", action_result);
        assert_eq!(action_result, Err(Error::InvalidPass));

        // Check PieceNotInHand error (try to place a piece not in hand)
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Rook,
                bottom: Piece::Rook,
            },
        );
        println!(
            "North tries to place Rook-Rook (not in hand): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::PieceNotInHand));

        // Cehck Normal place(first turn, so top can be placed face-down regardless of piece type)
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Lance,
            },
        );
        println!("North places Pawn-Lance on first turn: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "North's board after placing Pawn-Lance: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Down(Piece::Pawn),
                PieceWithFacing::Up(Piece::Lance),
            ]
        );

        // Chrck InvalidPlace error (try to place a piece different from the last placed piece
        // face-up)
        let action_result = game.play_turn(
            BoardDirection::East,
            PlayerAction::Place {
                top: Piece::Rook,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "East tries to place Rook-Pawn (top piece different from last placed Pawn and face-up): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::InvalidPlace));

        // Check Normal Place
        let action_result = game.play_turn(
            BoardDirection::East,
            PlayerAction::Place {
                top: Piece::Lance,
                bottom: Piece::Knight,
            },
        );
        println!("East places Lance-Knight: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "East's board after placing Lance-Knight: {:?}",
            game.player_board(BoardDirection::East)
        );
        assert_eq!(
            game.player_board(BoardDirection::East).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Lance),
                PieceWithFacing::Up(Piece::Knight),
            ]
        );

        // Check InvalidPlace error(Place king when not having 2 kings in hand and no
        // existing king on board)
        let action_result = game.play_turn(
            BoardDirection::South,
            PlayerAction::Place {
                top: Piece::Knight,
                bottom: Piece::King,
            },
        );
        println!(
            "South tries to place King-King (only has 1 King in hand and no existing King on board): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::InvalidPlace));

        // Normal place(Use King)
        let action_result = game.play_turn(
            BoardDirection::South,
            PlayerAction::Place {
                top: Piece::King,
                bottom: Piece::Pawn,
            },
        );
        println!("South places King-Pawn: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "South's board after placing King-Pawn: {:?}",
            game.player_board(BoardDirection::South)
        );
        assert_eq!(
            game.player_board(BoardDirection::South).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::King),
                PieceWithFacing::Up(Piece::Pawn),
            ]
        );

        // Check InvalidPlace error(try to place King when last placed piece is Pawn and top piece
        // is King)
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::King,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "West tries to place King-Pawn (top piece is King and last placed piece is Pawn): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::InvalidPlace));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::West, PlayerAction::Pass);
        println!("West passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Lance,
            },
        );
        println!("North places Pawn-Lance: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "North's board after placing Pawn-Lance: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Down(Piece::Pawn),
                PieceWithFacing::Up(Piece::Lance),
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Lance),
            ]
        );

        // Normal pass
        let action_result = game.play_turn(BoardDirection::East, PlayerAction::Pass);
        println!("East passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::South, PlayerAction::Pass);
        println!("South passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Check InvalidPlace(try to place King when last placed piece is Lance and top piece is
        // King)
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::King,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "West tries to place King-Pawn (top piece is King and last placed piece is Lance): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::InvalidPlace));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::West, PlayerAction::Pass);
        println!("West passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Invalid pass(try to pass when last placed player is the same as current player)
        let action_result = game.play_turn(BoardDirection::North, PlayerAction::Pass);
        println!(
            "North tries to pass (last placed player is the same as current player): {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::InvalidPass));

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Silver,
                bottom: Piece::Gold,
            },
        );
        println!("North places Silver-Gold: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "North's board after placing Silver-Gold: {:?}",
            game.player_board(BoardDirection::North),
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Down(Piece::Pawn),
                PieceWithFacing::Up(Piece::Lance),
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Lance),
                PieceWithFacing::Down(Piece::Silver),
                PieceWithFacing::Up(Piece::Gold),
            ]
        );

        // Normal pass
        let action_result = game.play_turn(BoardDirection::East, PlayerAction::Pass);
        println!("East passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::South, PlayerAction::Pass);
        println!("South passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal place(Place king to botton)
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Gold,
                bottom: Piece::King,
            },
        );
        println!("West places Gold-King: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "West's board after placing Gold-King: {:?}",
            game.player_board(BoardDirection::West),
        );
        assert_eq!(
            game.player_board(BoardDirection::West).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Gold),
                PieceWithFacing::Up(Piece::King),
            ]
        );

        // Normal pass
        let action_result = game.play_turn(BoardDirection::North, PlayerAction::Pass);
        println!("North passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::East, PlayerAction::Pass);
        println!("East passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::South, PlayerAction::Pass);
        println!("South passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // normal place
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Bishop,
                bottom: Piece::Silver,
            },
        );
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!("West places Bishop-Silver: {:?}", action_result);
        assert_eq!(
            game.player_board(BoardDirection::West).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Gold),
                PieceWithFacing::Up(Piece::King),
                PieceWithFacing::Down(Piece::Bishop),
                PieceWithFacing::Up(Piece::Silver),
            ]
        );

        // Normal pass
        let action_result = game.play_turn(BoardDirection::North, PlayerAction::Pass);
        println!("North passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::East, PlayerAction::Pass);
        println!("East passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::South, PlayerAction::Pass);
        println!("South passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // normal place
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!("West places Pawn-Pawn: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "West's board after placing Pawn-Pawn: {:?}",
            game.player_board(BoardDirection::West)
        );
        assert_eq!(
            game.player_board(BoardDirection::West).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Gold),
                PieceWithFacing::Up(Piece::King),
                PieceWithFacing::Down(Piece::Bishop),
                PieceWithFacing::Up(Piece::Silver),
                PieceWithFacing::Down(Piece::Pawn),
                PieceWithFacing::Up(Piece::Pawn),
            ]
        );

        // Normal pass
        let action_result = game.play_turn(BoardDirection::North, PlayerAction::Pass);
        println!("North passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::East, PlayerAction::Pass);
        println!("East passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal pass
        let action_result = game.play_turn(BoardDirection::South, PlayerAction::Pass);
        println!("South passes: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));

        // Normal place with end round(dobule up)
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!("West places Pawn-Pawn: {:?}", action_result);
        assert_eq!(
            action_result,
            Ok(ApplyResult::RoundOver(RoundResult::new(
                BoardDirection::West,
                20
            )))
        );

        // Check score
        println!(
            "Final Scores - North-South: {}, East-West: {}",
            game.ns_score(),
            game.ew_score()
        );
        assert_eq!(game.ns_score(), 0);
        assert_eq!(game.ew_score(), 20);

        // Check RoundIsOver error when trying to play a turn after round is over but before starting a new round
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!(
            "North tries to place Pawn-Pawn after round is over: {:?}",
            action_result
        );
        assert_eq!(action_result, Err(Error::RoundIsOver));

        // Start Round and check if the starting player is set correctly based on the previous round
        // result
        game.start_new_round().unwrap();
        println!(
            "New round started. Starting player should be West based on previous round result. Current turn player: {:?}",
            game.current_turn_plyer()
        );
        assert_eq!(game.current_turn_plyer(), Some(BoardDirection::West));

        println!(
            "Hands(North): {:?}",
            game.player_hand(BoardDirection::North)
        );
        assert_eq!(
            *game.player_hand(BoardDirection::North).unwrap(),
            hand! {
                Piece::Pawn   => 2,
                Piece::King   => 1,
                Piece::Knight => 1,
                Piece::Silver => 1,
                Piece::Lance  => 2,
                Piece::Gold   => 1,
            }
        );

        println!("Hands(East): {:?}", game.player_hand(BoardDirection::East));
        assert_eq!(
            *game.player_hand(BoardDirection::East).unwrap(),
            hand! {
                Piece::Knight => 1,
                Piece::Bishop => 2,
                Piece::Pawn   => 2,
                Piece::Lance  => 2,
                Piece::Rook   => 1,
            }
        );

        println!(
            "Hands(South): {:?}",
            game.player_hand(BoardDirection::South)
        );
        assert_eq!(
            *game.player_hand(BoardDirection::South).unwrap(),
            hand! {
                Piece::Pawn   => 3,
                Piece::Gold   => 2,
                Piece::Rook   => 1,
                Piece::Silver => 2,
            }
        );

        println!("Hands(West): {:?}", game.player_hand(BoardDirection::West));
        assert_eq!(
            *game.player_hand(BoardDirection::West).unwrap(),
            hand! {
                Piece::King   => 1,
                Piece::Silver => 1,
                Piece::Gold   => 1,
                Piece::Knight => 2,
                Piece::Pawn   => 3,
            }
        );

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::West,
            PlayerAction::Place {
                top: Piece::Knight,
                bottom: Piece::Pawn,
            },
        );
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!("West places Knight-Pawn: {:?}", action_result);
        assert_eq!(
            game.player_board(BoardDirection::West).unwrap(),
            vec![
                PieceWithFacing::Down(Piece::Knight),
                PieceWithFacing::Up(Piece::Pawn),
            ]
        );

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Pawn,
                bottom: Piece::Pawn,
            },
        );
        println!("North places Pawn-Pawn: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "North's board after placing Pawn-Pawn: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Pawn),
            ]
        );

        pass_until_last_placed_player(&mut game);

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Knight,
                bottom: Piece::Silver,
            },
        );
        println!("North places Knight-Silver: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "West's board after placing Knight-Silver: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Down(Piece::Knight),
                PieceWithFacing::Up(Piece::Silver),
            ]
        );

        pass_until_last_placed_player(&mut game);

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Lance,
                bottom: Piece::Lance,
            },
        );
        println!("North places Lance-Lance: {:?}", action_result);
        assert_eq!(action_result, Ok(ApplyResult::Continuing));
        println!(
            "North's board after placing Lance-Lance: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Down(Piece::Knight),
                PieceWithFacing::Up(Piece::Silver),
                PieceWithFacing::Down(Piece::Lance),
                PieceWithFacing::Up(Piece::Lance),
            ]
        );

        pass_until_last_placed_player(&mut game);

        // Normal place
        let action_result = game.play_turn(
            BoardDirection::North,
            PlayerAction::Place {
                top: Piece::Gold,
                bottom: Piece::King,
            },
        );
        println!("North places Gold-King: {:?}", action_result);
        assert_eq!(
            action_result,
            Ok(ApplyResult::RoundOver(RoundResult::new(
                BoardDirection::North,
                50,
            )))
        );
        println!(
            "North's board after placing Gold-King: {:?}",
            game.player_board(BoardDirection::North)
        );
        assert_eq!(
            game.player_board(BoardDirection::North).unwrap(),
            vec![
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Up(Piece::Pawn),
                PieceWithFacing::Down(Piece::Knight),
                PieceWithFacing::Up(Piece::Silver),
                PieceWithFacing::Down(Piece::Lance),
                PieceWithFacing::Up(Piece::Lance),
                PieceWithFacing::Down(Piece::Gold),
                PieceWithFacing::Up(Piece::King),
            ]
        );

        // Check Error GameIsOver when trying to start new round after game is over
        let round_result = game.start_new_round();
        println!(
            "Trying to start new round after game is over: {:?}",
            round_result
        );
        assert_eq!(round_result, Err(Error::GameIsOver));

        // Check game result
        let game_result = game.check_game_over();
        println!(
            "Game result after North reaches winning score: {:?}",
            game_result
        );
        assert_eq!(game_result, Some(GameResult::new(Team::NorthSouth, 50, 20)));
    }

    // 最後に駒をおいたプレイヤーまでパスするヘルパー関数
    fn pass_until_last_placed_player(game: &mut GoitaGame) {
        for _ in 0..3 {
            // Normal pass
            let player = game.current_turn_plyer().unwrap();
            let action_result = game.play_turn(player, PlayerAction::Pass);
            if let Err(Error::InvalidPass) = action_result {
                // If the pass is invalid, it means the last placed player is the same as the current player,
                // so we break the loop to avoid infinite loop.
                break;
            }
            println!("{:?} passes: {:?}", player, action_result);
        }
    }
}
