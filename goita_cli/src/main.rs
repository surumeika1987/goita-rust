use goita::*;
use std::io;

fn main() {
    println!("Goita CLI");
    println!("type 'help' for a list of commands, 'exit' to quit.");
    let mut game: Option<GoitaGame> = None;

    // main loop
    loop {
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        let commands = command.trim().split_whitespace().collect::<Vec<&str>>();

        if let Some(command) = commands.first() {
            match *command {
                "help" => {
                    if let Some(arg) = commands.get(1) {
                        match *arg {
                            "sg" => println!("sg - Start a new game"),
                            "nr" => println!("nr - Start a new round"),
                            "pp" => {
                                println!("pp <piece><piece> - play a piece (e.g., 'pp kp')");
                                println!("Available pieces:");
                                println!("k - King(王、玉)");
                                println!("r - Rook(飛)");
                                println!("b - Bishop(角)");
                                println!("g - Gold(金)");
                                println!("s - silver(銀)");
                                println!("n - Knight(馬)");
                                println!("l - Lance(香)");
                                println!("p - Pawn(し)");
                            }
                            "pa" => println!("pa - pass"),
                            _ => println!("Unknown command: {}", arg),
                        }
                    } else {
                        println!("Available commands:");
                        println!("sg - Start a new game");
                        println!("nr - Start a new round");
                        println!("pp <piece><piece> - play a piece (e.g., 'pp kp')");
                        println!("pa - pass");
                        println!("help <command> - Show command usage");
                        println!("exit - Exit the program");
                    }
                }
                "sg" => {
                    game = Some(GoitaGame::new(GoitaRule::default(), BoardDirection::North));
                }
                "nr" => {
                    if let Some(ref mut game) = game {
                        game.start_new_round();
                        println!("New round started.");
                        print_board(game);
                        print_hand(game, game.current_turn_player().unwrap());
                    } else {
                        println!("No game in progress. Use 'sg' to start a new game.");
                    }
                }
                "pp" => {
                    if let Some(arg) = commands.get(1) {
                        if let Some(game) = game.as_mut() {
                            if arg.len() == 2 {
                                let piece1 = convert_str_to_piece(&arg[0..1]);
                                let piece2 = convert_str_to_piece(&arg[1..2]);
                                if let (Some(p1), Some(p2)) = (piece1, piece2) {
                                    if let Some(current_trun_player) = game.current_turn_player() {
                                        match game.play_turn(
                                            current_trun_player,
                                            PlayerAction::Place {
                                                top: p1,
                                                bottom: p2,
                                            },
                                        ) {
                                            Ok(result) => match result {
                                                ApplyResult::Continuing => {
                                                    println!(
                                                        "Place applied. Next player is {:?}",
                                                        current_trun_player.next()
                                                    );
                                                    print_board(game);
                                                    print_hand(game, current_trun_player.next());
                                                }
                                                ApplyResult::RoundOver(result) => {
                                                    println!(
                                                        "Round over. Won team: {:?}, score: {:?}",
                                                        result.winning_team(),
                                                        result.score()
                                                    );
                                                    if let Some(game_result) =
                                                        game.check_game_over()
                                                    {
                                                        println!(
                                                            "Game over. Won team: {:?}, ns_score: {:?}, ew_score: {:?}",
                                                            game_result.winning_team(),
                                                            game_result.north_south_score()
                                                            game_result.east_west_score()
                                                        );
                                                    }
                                                }
                                            },
                                            Err(err) => {
                                                println!("Failed to apply action: {:?}", err)
                                            }
                                        }
                                    } else {
                                        println!(
                                            "Round is not active. Use 'nr' to start a new round."
                                        );
                                    }
                                } else {
                                    println!("Invalid pieces. Use 'help pp' for available pieces.");
                                }
                            } else {
                                println!(
                                    "Invalid command format. Use 'pp <piece><piece>' (e.g., 'pp kp')."
                                );
                            }
                        } else {
                            println!("No game in progress. Use 'sg' to start a new game.");
                        }
                    } else {
                        println!("No pieces specified. Use 'pp <piece><piece>' (e.g., 'pp kp').");
                    }
                }
                "pa" => {
                    if let Some(game) = game.as_mut() {
                        if let Some(current_trun_player) = game.current_turn_player() {
                            match game.play_turn(current_trun_player, PlayerAction::Pass) {
                                Ok(result) => match result {
                                    ApplyResult::Continuing => {
                                        println!(
                                            "Pass applied. Next player is {:?}",
                                            current_trun_player.next()
                                        );
                                        print_board(game);
                                        print_hand(game, current_trun_player.next());
                                    }
                                    ApplyResult::RoundOver(result) => {
                                        println!(
                                            "Round over. Won team: {:?}, score: {:?}",
                                            result.winning_team(),
                                            result.score()
                                        );
                                    }
                                },
                                Err(err) => println!("Failed to apply action: {:?}", err),
                            }
                        } else {
                            println!("Round is not active. Use 'nr' to start a new round.");
                        }
                    } else {
                        println!("No game in progress. Use 'sg' to start a new game.");
                    }
                }
                "exit" | "quit" => {
                    println!("Exiting...");
                    break;
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        } else {
            println!("No command entered.");
        }
    }
}

fn print_board(game: &GoitaGame) {
    for i in 0..4 {
        let player = BoardDirection::from(i);
        let board = game.player_board(player).unwrap_or_default();
        let mut board_str = String::new();
        for piece in board {
            match piece {
                PieceWithFacing::FaceUp(piece) => board_str += convert_piece_to_str(piece),
                PieceWithFacing::FaceDown(_) => board_str += "x",
            }
        }
        println!("Player({:?}) Board: {}", player, board_str);
    }
}

fn print_hand(game: &GoitaGame, player: BoardDirection) {
    let hand = game.player_hand(player).unwrap_or_default();
    let mut hand_str = String::new();
    for piece in hand {
        hand_str += convert_piece_to_str(piece);
    }
    println!("Player({:?}) Hand: {}", player, hand_str);
}

fn convert_piece_to_str(piece: Piece) -> &'static str {
    match piece {
        Piece::King => "k",
        Piece::Rook => "r",
        Piece::Bishop => "b",
        Piece::Gold => "g",
        Piece::Silver => "s",
        Piece::Knight => "n",
        Piece::Lance => "l",
        Piece::Pawn => "p",
    }
}

fn convert_str_to_piece(s: &str) -> Option<Piece> {
    match s {
        "k" => Some(Piece::King),
        "r" => Some(Piece::Rook),
        "b" => Some(Piece::Bishop),
        "g" => Some(Piece::Gold),
        "s" => Some(Piece::Silver),
        "n" => Some(Piece::Knight),
        "l" => Some(Piece::Lance),
        "p" => Some(Piece::Pawn),
        _ => None,
    }
}
