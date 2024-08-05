use game_session_io::*;
use gtest::{Log, ProgramBuilder, System};

const GAME_SESSION_PROGRAM_ID: u64 = 1;
const WORDLE_PROGRAM_ID: u64 = 2;

const USER: u64 = 3;

#[test]
#[ignore]
fn test_win() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    let result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!result.main_failed());

    // CheckWord failed: The user is not in the game
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "abcde".to_string(),
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload_bytes("Panic occurred: panicked with 'The user is not in the game'");
    assert!(result.main_failed() && result.contains(&error_log));

    // StartGame success
    let result = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!result.main_failed() && result.contains(&log));

    // StartGame failed: The user is in the game
    let result = game_session_program.send(USER, GameSessionAction::StartGame);
    let error_log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload_bytes("Panic occurred: panicked with 'The user is in the game'");
    assert!(result.main_failed() && result.contains(&error_log));

    // CheckWord failed: Invalid word
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "Abcde".to_string(),
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload_bytes("Panic occurred: panicked with 'Invalid word'");
    assert!(result.main_failed() && result.contains(&error_log));
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "abcdef".to_string(),
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload_bytes("Panic occurred: panicked with 'Invalid word'");
    assert!(result.main_failed() && result.contains(&error_log));

    // CheckWord success, but not guessed
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "house".to_string(),
        },
    );
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::CheckWordResult {
            correct_positions: vec![0, 1, 3, 4],
            contained_in_word: vec![],
        });
    assert!(!result.main_failed() && result.contains(&log));

    // CheckWord success and has been  guessed
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "horse".to_string(),
        },
    );
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Win));
    assert!(!result.main_failed() && result.contains(&log));

    // CheckWord failed: The user is not in the game
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "abcde".to_string(),
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload_bytes("Panic occurred: panicked with 'The user is not in the game'");
    assert!(result.main_failed() && result.contains(&error_log));

    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}

#[test]
fn test_lose_exceeded_tries_limit() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    let result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!result.main_failed());

    // StartGame success
    let result = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!result.main_failed() && result.contains(&log));

    for i in 0..5 {
        // CheckWord success, but not guessed
        let result = game_session_program.send(
            USER,
            GameSessionAction::CheckWord {
                word: "house".to_string(),
            },
        );
        if i == 4 {
            let log = Log::builder()
                .dest(USER)
                .source(GAME_SESSION_PROGRAM_ID)
                .payload(GameSessionEvent::GameOver(GameStatus::Lose));
            assert!(!result.main_failed() && result.contains(&log));
        } else {
            let log = Log::builder()
                .dest(USER)
                .source(GAME_SESSION_PROGRAM_ID)
                .payload(GameSessionEvent::CheckWordResult {
                    correct_positions: vec![0, 1, 3, 4],
                    contained_in_word: vec![],
                });
            assert!(!result.main_failed() && result.contains(&log));
        }
    }
    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}

#[test]
#[ignore]
fn test_lose_timeout() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    let result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!result.main_failed());

    // StartGame success
    let result = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!result.main_failed() && result.contains(&log));

    let result = system.spend_blocks(200);
    println!("{:?}", result);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Lose));
    assert!(result[0].contains(&log));
    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}
