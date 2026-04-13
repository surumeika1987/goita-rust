use goita::*;

mod common;

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
    let mut game = GoitaGame::new_with_seed(GoitaRule::new(50), BoardDirection::North, 12345);

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
            Piece::Gold => 2,
            Piece::Silver => 1,
            Piece::Lance => 3,
            Piece::Pawn => 2,
        }
    );

    println!("Hands(East): {:?}", game.player_hand(BoardDirection::East));
    assert_eq!(
        *game.player_hand(BoardDirection::East).unwrap(),
        hand! {
            Piece::Rook   => 1,
            Piece::Bishop => 1,
            Piece::Silver => 1,
            Piece::Knight => 3,
            Piece::Lance  => 1,
            Piece::Pawn   => 1,
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
            Piece::Gold   => 1,
            Piece::Silver => 1,
            Piece::Knight => 1,
            Piece::Pawn   => 3,
        }
    );

    println!("Hands(West): {:?}", game.player_hand(BoardDirection::West));
    assert_eq!(
        *game.player_hand(BoardDirection::West).unwrap(),
        hand! {
            Piece::King   => 1,
            Piece::Bishop => 1,
            Piece::Gold   => 1,
            Piece::Silver => 1,
            Piece::Pawn   => 4,
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
            PieceWithFacing::FaceDown(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Lance),
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
    assert_eq!(
        action_result,
        Err(Error::InvalidPlace(InvalidPlaceError::PieceMismatch {
            expected: Piece::Lance,
            actual: Piece::Rook
        }))
    );

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
            PieceWithFacing::FaceUp(Piece::Lance),
            PieceWithFacing::FaceUp(Piece::Knight),
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
        "Sourh tries to place Knight-King (try to place King when not having 2 kings in hand and no existing king on board): {:?}",
        action_result
    );
    assert_eq!(
        action_result,
        Err(Error::InvalidPlace(InvalidPlaceError::InvalidKingPlacement))
    );

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
            PieceWithFacing::FaceUp(Piece::King),
            PieceWithFacing::FaceUp(Piece::Pawn),
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
    assert_eq!(
        action_result,
        Err(Error::InvalidPlace(InvalidPlaceError::PieceMismatch {
            expected: Piece::Pawn,
            actual: Piece::King
        }))
    );

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
            PieceWithFacing::FaceDown(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Lance),
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Lance),
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
    assert_eq!(
        action_result,
        Err(Error::InvalidPlace(InvalidPlaceError::PieceMismatch {
            expected: Piece::Lance,
            actual: Piece::King
        }))
    );

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
            PieceWithFacing::FaceDown(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Lance),
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Lance),
            PieceWithFacing::FaceDown(Piece::Silver),
            PieceWithFacing::FaceUp(Piece::Gold),
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
            PieceWithFacing::FaceUp(Piece::Gold),
            PieceWithFacing::FaceUp(Piece::King),
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
            PieceWithFacing::FaceUp(Piece::Gold),
            PieceWithFacing::FaceUp(Piece::King),
            PieceWithFacing::FaceDown(Piece::Bishop),
            PieceWithFacing::FaceUp(Piece::Silver),
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
            PieceWithFacing::FaceUp(Piece::Gold),
            PieceWithFacing::FaceUp(Piece::King),
            PieceWithFacing::FaceDown(Piece::Bishop),
            PieceWithFacing::FaceUp(Piece::Silver),
            PieceWithFacing::FaceDown(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Pawn),
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
        game.current_turn_player()
    );
    assert_eq!(game.current_turn_player(), Some(BoardDirection::West));

    println!(
        "Hands(North): {:?}",
        game.player_hand(BoardDirection::North)
    );
    assert_eq!(
        *game.player_hand(BoardDirection::North).unwrap(),
        hand! {
            Piece::King   => 1,
            Piece::Gold   => 1,
            Piece::Silver => 1,
            Piece::Knight => 1,
            Piece::Lance  => 2,
            Piece::Pawn   => 2,
        }
    );

    println!("Hands(East): {:?}", game.player_hand(BoardDirection::East));
    assert_eq!(
        *game.player_hand(BoardDirection::East).unwrap(),
        hand! {
            Piece::Rook   => 1,
            Piece::Bishop => 2,
            Piece::Knight => 1,
            Piece::Lance  => 2,
            Piece::Pawn   => 2,
        }
    );

    println!(
        "Hands(South): {:?}",
        game.player_hand(BoardDirection::South)
    );
    assert_eq!(
        *game.player_hand(BoardDirection::South).unwrap(),
        hand! {
            Piece::Rook   => 1,
            Piece::Gold   => 2,
            Piece::Silver => 2,
            Piece::Pawn   => 3,
        }
    );

    println!("Hands(West): {:?}", game.player_hand(BoardDirection::West));
    assert_eq!(
        *game.player_hand(BoardDirection::West).unwrap(),
        hand! {
            Piece::King   => 1,
            Piece::Gold   => 1,
            Piece::Silver => 1,
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
            PieceWithFacing::FaceDown(Piece::Knight),
            PieceWithFacing::FaceUp(Piece::Pawn),
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
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Pawn),
        ]
    );

    common::pass_until_last_placed_player(&mut game);

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
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceDown(Piece::Knight),
            PieceWithFacing::FaceUp(Piece::Silver),
        ]
    );

    common::pass_until_last_placed_player(&mut game);

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
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceDown(Piece::Knight),
            PieceWithFacing::FaceUp(Piece::Silver),
            PieceWithFacing::FaceDown(Piece::Lance),
            PieceWithFacing::FaceUp(Piece::Lance),
        ]
    );

    common::pass_until_last_placed_player(&mut game);

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
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceUp(Piece::Pawn),
            PieceWithFacing::FaceDown(Piece::Knight),
            PieceWithFacing::FaceUp(Piece::Silver),
            PieceWithFacing::FaceDown(Piece::Lance),
            PieceWithFacing::FaceUp(Piece::Lance),
            PieceWithFacing::FaceDown(Piece::Gold),
            PieceWithFacing::FaceUp(Piece::King),
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
