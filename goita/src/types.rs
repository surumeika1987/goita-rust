use goita_core::{BoardDirection, Piece, Team};

/// Errors that can occur while processing a game action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// The specified hand is invalid (e.g. wrong number of pieces, contains pieces that don't exist
    /// in the game, etc.).
    InvalidHand,
    /// The GameNotStarted.
    GameNotStarted,
    /// The action was attempted by a player who is not the current turn player.
    NotYourTurn,
    /// The specified piece is not available in the player's hand.
    PieceNotInHand,
    /// The piece placement is not valid under the game rules.
    // on bottom if certain conditions are not met", etc.)
    InvalidPlace(InvalidPlaceError),
    /// The pass action is not valid in the current game state.
    InvalidPass,
    /// The round has already ended, so no further actions are allowed.
    RoundIsOver,
    /// The game has already ended, so no further actions and start new rounds are allowed.
    GameIsOver,
}

/// Errors that can occur when attempting to place a piece on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidPlaceError {
    /// A face-up piece cannot be placed in this position.
    FaceUpNotAllowed,
    /// A face-down piece cannot be placed in this position.
    FaceDownNotAllowed,
    /// The placed piece does not match the expected piece for the target position.
    PieceMismatch { expected: Piece, actual: Piece },
    /// A king was placed in a position where king placement is not allowed.
    InvalidKingPlacement,
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
