use crate::{ApplyResult, DealEvent, Error, GameResult, GoitaRound, GoitaRule, HandRank};
use goita_core::{BoardDirection, Piece, PieceWithFacing, PlayerAction, Team};
use rand::prelude::*;

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
        if self.check_game_over().is_some() {
            return Err(Error::GameIsOver);
        }
        let mut round = GoitaRound::new(self.round_start_player);
        let seed: u64 = self.rng.random();
        let deal_event = round.shuffle_and_deal_hands(seed);
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
    /// # Errors
    ///
    /// Returns:
    /// - `Error::GameNotStarted` if there is no active round,
    /// - `Error::NotYourTurn` if the action is attempted by a player who is not the current turn player,
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
        if self.check_game_over().is_some() {
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

    pub fn current_turn_player(&self) -> Option<BoardDirection> {
        if self.check_game_over().is_some() {
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

    /// Validates whether a player can place the specified pieces in the current game state.
    ///
    /// This method checks:
    /// - the game has started,
    /// - the game is not already over,
    /// - the current round is not over,
    /// - and the move itself is valid according to round rules.
    ///
    /// # Parameters
    /// - `player`: The direction/player attempting to place pieces.
    /// - `top_piece`: The top piece with facing information.
    /// - `bottom_piece`: The bottom piece to place.
    ///
    /// # Returns
    /// - `Ok(())` if the placement is valid.
    /// - `Err(Error::GameIsOver)` if the game has already ended.
    /// - `Err(Error::RoundIsOver)` if the current round has ended.
    /// - `Err(Error::InvalidPlace(_))` if the move violates placement rules.
    /// - `Err(Error::GameNotStarted)` if no round is currently active.
    pub fn check_place_piece(
        &self,
        player: BoardDirection,
        top_piece: PieceWithFacing,
        bottom_piece: Piece,
    ) -> Result<(), Error> {
        if let Some(_) = self.check_game_over() {
            return Err(Error::GameIsOver);
        }
        if let Some(round) = &self.current_round {
            if round.round_is_over() {
                return Err(Error::RoundIsOver);
            }
            match round.check_place_pieces(player, top_piece, bottom_piece) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::InvalidPlace(e)),
            }
        } else {
            return Err(Error::GameNotStarted);
        }
    }

    /// Returns the hand of the specified player in the current round.
    ///
    /// If no round is currently active, this returns `None`.
    /// Otherwise, it returns `Some(&Hand)` for the given `player`.
    pub fn player_hand(&self, player: BoardDirection) -> Option<Vec<Piece>> {
        self.current_round
            .as_ref()
            .map(|round| round.player_hand(player).pieces())
    }

    /// Returns the current board state for the specified player in the active round.
    ///
    /// If no round is currently active, this returns `None`.
    pub fn player_board(&self, player: BoardDirection) -> Option<Vec<PieceWithFacing>> {
        self.current_round
            .as_ref()
            .map(|round| round.player_board(player))
    }

    /// Returns the most recently placed piece in the current round.
    ///
    /// If there is no active round or no piece has been placed yet, this returns `None`.
    pub fn last_placed_piece(&self) -> Option<Piece> {
        self.current_round
            .as_ref()
            .and_then(|round| round.last_placed_piece())
    }
}
