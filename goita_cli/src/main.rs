use goita::*;
use std::io;

fn main() {
    println!("Goita CLI");
    println!("type 'help' for a list of commands, 'exit' to quit.");
    let mut game: Option<GoitaGame> = None;

    loop {
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        let commands = command.split_whitespace().collect::<Vec<&str>>();
        let Some(command) = commands.first() else {
            println!("No command entered.");
            continue;
        };

        match *command {
            "help" => print_help(commands.get(1).copied()),
            "sg" => start_game(&mut game),
            "nr" => start_new_round(&mut game),
            "pp" => play_piece(&mut game, commands.get(1).copied()),
            "pa" => pass_turn(&mut game),
            "exit" | "quit" => {
                println!("Exiting...");
                break;
            }
            _ => println!("Unknown command: {}", command),
        }
    }
}

fn print_help(command: Option<&str>) {
    match command {
        Some("sg") => println!("sg - Start a new game"),
        Some("nr") => println!("nr - Start a new round"),
        Some("pp") => {
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
        Some("pa") => println!("pa - pass"),
        Some(arg) => println!("Unknown command: {}", arg),
        None => {
            println!("Available commands:");
            println!("sg - Start a new game");
            println!("nr - Start a new round");
            println!("pp <piece><piece> - play a piece (e.g., 'pp kp')");
            println!("pa - pass");
            println!("help <command> - Show command usage");
            println!("exit - Exit the program");
        }
    }
}

fn start_game(game: &mut Option<GoitaGame>) {
    *game = Some(GoitaGame::new(GoitaRule::default(), BoardDirection::North));
}

fn start_new_round(game: &mut Option<GoitaGame>) {
    let Some(game) = game.as_mut() else {
        println!("No game in progress. Use 'sg' to start a new game.");
        return;
    };

    if let Err(err) = game.start_new_round() {
        println!("Failed to start new round: {:?}", err);
        return;
    }
    println!("New round started.");
    print_board(game);

    if let Some(current_turn_player) = game.current_turn_player() {
        print_hand(game, current_turn_player);
    }
}

fn play_piece(game: &mut Option<GoitaGame>, piece_arg: Option<&str>) {
    let Some(piece_arg) = piece_arg else {
        println!("No pieces specified. Use 'pp <piece><piece>' (e.g., 'pp kp').");
        return;
    };

    let Some(game) = game.as_mut() else {
        println!("No game in progress. Use 'sg' to start a new game.");
        return;
    };

    if piece_arg.len() != 2 {
        println!("Invalid command format. Use 'pp <piece><piece>' (e.g., 'pp kp').");
        return;
    }

    // 入力は2文字固定（例: "kp"）のため、1文字ずつ駒へ変換する。
    let top = convert_str_to_piece(&piece_arg[0..1]);
    let bottom = convert_str_to_piece(&piece_arg[1..2]);
    let (Some(top), Some(bottom)) = (top, bottom) else {
        println!("Invalid pieces. Use 'help pp' for available pieces.");
        return;
    };

    apply_action(game, PlayerAction::Place { top, bottom }, "Place", true);
}

fn pass_turn(game: &mut Option<GoitaGame>) {
    let Some(game) = game.as_mut() else {
        println!("No game in progress. Use 'sg' to start a new game.");
        return;
    };

    apply_action(game, PlayerAction::Pass, "Pass", false);
}

fn apply_action(
    game: &mut GoitaGame,
    action: PlayerAction,
    action_name: &str,
    check_game_over: bool,
) {
    // 手番適用後は状態が進むため、表示に使う手番は先に保持しておく。
    let Some(current_turn_player) = game.current_turn_player() else {
        println!("Round is not active. Use 'nr' to start a new round.");
        return;
    };

    match game.play_turn(current_turn_player, action) {
        Ok(ApplyResult::Continuing) => {
            println!(
                "{} applied. Next player is {:?}",
                action_name,
                current_turn_player.next()
            );
            print_board(game);
            print_hand(game, current_turn_player.next());
        }
        Ok(ApplyResult::RoundOver(result)) => {
            println!(
                "Round over. Won team: {:?}, score: {:?}",
                result.winning_team(),
                result.score()
            );

            if check_game_over {
                if let Some(game_result) = game.check_game_over() {
                    println!(
                        "Game over. Won team: {:?}, ns_score: {:?}, ew_score: {:?}",
                        game_result.winning_team(),
                        game_result.north_south_score(),
                        game_result.east_west_score()
                    );
                }
            }
        }
        Err(err) => println!("Failed to apply action: {:?}", err),
    }
}

fn print_board(game: &GoitaGame) {
    for i in 0..4 {
        let player = BoardDirection::from(i);
        let board = game.player_board(player).unwrap_or_default();
        let mut board_str = String::new();

        for piece in board {
            match piece {
                PieceWithFacing::FaceUp(piece) => board_str.push_str(convert_piece_to_str(piece)),
                PieceWithFacing::FaceDown(_) => board_str.push('x'),
            }
        }

        println!("Player({:?}) Board: {}", player, board_str);
    }
}

fn print_hand(game: &GoitaGame, player: BoardDirection) {
    let hand = game.player_hand(player).unwrap_or_default();
    let mut hand_str = String::new();

    for piece in hand {
        hand_str.push_str(convert_piece_to_str(piece));
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
