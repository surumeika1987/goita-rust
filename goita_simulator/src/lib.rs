use goita_core::*;
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
    InvalidPlace,
    /// The pass action is not valid in the current game state.
    InvalidPass,
    /// The round has already ended, so no further actions are allowed.
    RoundIsOver,
    /// The game has already ended, so no further actions and start new rounds are allowed.
    GameIsOver,
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
    pub fn winning_team(&self) -> &Team {
        &self.winning_team
    }

    pub fn winning_player(&self) -> BoardDirection {
        self.last_place_player
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
    /// The Game has ended with a final result.
    GameOver(GameResult),
}

// 1ラウンド分の進行状態を保持する構造体。
struct GoitaRound {
    // 現在の盤面
    board: Board,
    // 現在の手番プレイヤー
    current_player: BoardDirection,
    // 最後に駒を置いたプレイヤー（未配置なら None）
    last_place_player: Option<BoardDirection>,
    // 4人分の手札
    hands: [Hand; 4],
    // ラウンド終了フラグ
    round_is_over: bool,
}

impl GoitaRound {
    // 指定した開始手番プレイヤーでラウンドを初期化し、
    // 駒をシャッフルして各プレイヤーに配る。
    pub fn new(init_turn_player: BoardDirection) -> Self {
        let mut round = Self {
            board: Board::new(),
            current_player: init_turn_player,
            last_place_player: None,
            hands: [Hand::new(), Hand::new(), Hand::new(), Hand::new()],
            round_is_over: false,
        };

        round.shuffle_and_deal_hands();
        round
    }

    // 駒の既定構成を展開してシャッフルし、8枚ずつ4人分の手札として配る。
    // - `DEFAULT_PIECES` の (駒種, 枚数) から全32枚の配列を生成
    // - 乱数で順序をランダム化
    // - 8枚ごとに `Hand` を作成し、4人分の `self.hands` に格納
    // - 32枚ちょうどで4分割できる前提のため `try_into().unwrap()` を使用
    pub fn shuffle_and_deal_hands(&mut self) {
        const DEFAULT_PIECES: [(Piece, u8); 8] = [
            (Piece::King, 2),
            (Piece::Rook, 2),
            (Piece::Bishop, 2),
            (Piece::Gold, 4),
            (Piece::Silver, 4),
            (Piece::Knight, 4),
            (Piece::Lance, 4),
            (Piece::Pawn, 10),
        ];

        let mut expanded: Vec<Piece> = DEFAULT_PIECES
            .iter()
            .flat_map(|(piece, count)| std::iter::repeat(*piece).take(*count as usize))
            .collect();

        let mut rng = rand::rng();
        expanded.shuffle(&mut rng);

        self.hands = expanded
            .chunks(8)
            .map(|chunk| Hand::from(chunk.to_vec()))
            .collect::<Vec<Hand>>()
            .try_into()
            .unwrap()
    }

    // 指定したプレイヤーの手札を参照として返す。
    pub fn get_player_hand(&self, player: BoardDirection) -> &Hand {
        &self.hands[player as usize]
    }

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
        if player != self.current_player {
            return Err(Error::NotYourTurn);
        }
        if self.round_is_over {
            return Err(Error::RoundIsOver);
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
        Ok(round_result)
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
        self.current_player = self.current_player.next();
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
    // 上記以外は配置不可。
    fn can_place_king(&self, player: BoardDirection) -> bool {
        if self.hands[player as usize].count(Piece::King) == 2 {
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
pub struct GoitaGame {
    /// game rule configuration, which can be extended in the future to include more parameters if
    /// needed.
    game_rule: GoitaRule,
    // Cumulative score for the North-South team.
    ns_score: u32,
    // Cumulative score for the East-West team.
    ew_score: u32,
    // Round state currently in progress.
    current_round: Option<GoitaRound>,
    // Starting player for the next round, which is determined by the previous round's result.
    round_start_player: BoardDirection,
    // Game over flag to indicate if the game has ended, which can be used to prevent further
    // actions after a game result is reached.
    game_over: bool,
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
        Self {
            game_rule,
            ns_score: 0,
            ew_score: 0,
            current_round: None,
            round_start_player: initial_round_start_player,
            game_over: false,
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
    /// - `Ok(())` if a new round is successfully started.
    /// - `Err(Error::GameIsOver)` if the game has already ended.
    pub fn start_new_round(&mut self) -> Result<(), Error> {
        if game_over {
            return Err(Error::GameIsOver);
        }
        let mut round = GoitaRound::new(self.round_start_player);
        round.shuffle_and_deal_hands();
        self.current_round = Some(round);
        Ok(())
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
        if game_over {
            return Err(Error::GameIsOver);
        }
        if let Some(round) = &mut self.current_round {
            let result = round.action(player, action)?;
            if let ApplyResult::RoundOver(round_result) = result {
                match round_result.winning_team() {
                    Team::NorthSouth => self.ns_score += round_result.score(),
                    Team::EastWest => self.ew_score += round_result.score(),
                }

                if self.game_rule.winning_score() <= self.score(*round_result.winning_team()) {
                    let game_result =
                        GameResult::new(*round_result.winning_team(), self.ns_score, self.ew_score);
                    self.game_over = true;
                    return Ok(ApplyResult::GameOver(game_result));
                }

                self.round_start_player = round_result.winning_team().first_player_direction();
            }

            Ok(result)
        } else {
            Err(Error::GameNotStarted)
        }
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
}
