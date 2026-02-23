import random

import spooky_connect4


def test_game_initial_state() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    assert game.turn() == spooky_connect4.RED
    assert not game.is_over()
    assert game.outcome() is None


def test_game_legal_moves_initial() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    moves = game.legal_moves()
    assert len(moves) == 7  # All 7 columns are available


def test_game_make_move() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    moves = game.legal_moves()
    assert len(moves) > 0

    move = moves[0]
    success = game.make_move(move)
    assert success
    assert game.turn() == spooky_connect4.YELLOW


def test_game_unmake_1_move() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    moves = game.legal_moves()
    move = moves[0]

    game.make_move(move)
    assert game.turn() == spooky_connect4.YELLOW

    success = game.unmake_move()
    assert success
    assert game.turn() == spooky_connect4.RED


def test_game_unmake_10_moves() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Make 10 moves
    moves = []
    for _ in range(10):
        legal_moves = game.legal_moves()
        if not legal_moves:
            break

        move = legal_moves[0]
        moves.append(move)
        game.make_move(move)

    # Unmake all moves
    for _ in range(len(moves)):
        success = game.unmake_move()
        assert success

    # Should be back to initial state
    assert game.turn() == spooky_connect4.RED
    assert not game.is_over()
    assert len(game.legal_moves()) == 7


def test_game_make_invalid_move_invalid_column() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    # Create a move with invalid column
    invalid_move = spooky_connect4.Move(10, 0)
    success = game.make_move(invalid_move)
    assert not success


def test_game_make_invalid_move_full_column() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Fill column 0 completely
    for i in range(6):
        game.make_move(spooky_connect4.Move(0, i))

    # Try to add another piece to column 0
    board = game.board()
    assert board.is_column_full(0)

    # This move should be rejected (wrong row)
    invalid_move = spooky_connect4.Move(0, 0)
    assert not game.is_legal_move(invalid_move)


def test_game_column_fill() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Fill column 0 completely (6 moves)
    for i in range(6):
        move = spooky_connect4.Move(0, i)
        success = game.make_move(move)
        assert success

    # Column 0 should now be full
    board = game.board()
    assert board.is_column_full(0)


def test_game_vertical_win() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Red plays column 0 four times
    for i in range(3):
        # Red move
        red_move = spooky_connect4.Move(0, i)
        game.make_move(red_move)

        # Yellow move in different column
        yellow_move = spooky_connect4.Move(1, i)
        game.make_move(yellow_move)

    # Red's fourth move should win
    winning_move = spooky_connect4.Move(0, 3)
    game.make_move(winning_move)

    assert game.is_over()
    outcome = game.outcome()
    assert outcome is not None
    assert outcome.winner() == spooky_connect4.RED
    assert not outcome.is_draw()


def test_game_horizontal_win() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Create horizontal win for Red
    # Red: columns 0, 1, 2, 3 (row 0)
    # Yellow: columns 0, 1, 2 (row 1)

    for col in range(3):
        # Red plays bottom row
        red_move = spooky_connect4.Move(col, 0)
        game.make_move(red_move)

        # Yellow plays second row
        yellow_move = spooky_connect4.Move(col, 1)
        game.make_move(yellow_move)

    # Red's fourth move should win
    winning_move = spooky_connect4.Move(3, 0)
    game.make_move(winning_move)

    assert game.is_over()
    outcome = game.outcome()
    assert outcome is not None
    assert outcome.winner() == spooky_connect4.RED


def test_game_clone() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    move = game.legal_moves()[0]
    game.make_move(move)

    cloned = game.clone()
    assert cloned.turn() == game.turn()
    assert cloned.is_over() == game.is_over()


def test_game_is_legal_move() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Legal move
    legal_move = spooky_connect4.Move(0, 0)
    assert game.is_legal_move(legal_move)

    # Make the move
    game.make_move(legal_move)

    # Next piece in column 0 should be at row 1
    next_move = spooky_connect4.Move(0, 1)
    assert game.is_legal_move(next_move)

    # Row 0 is no longer legal in column 0
    assert not game.is_legal_move(legal_move)


def test_full_game() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    moves_made = 0
    max_moves = 42  # Maximum possible moves in Connect4

    while not game.is_over() and moves_made < max_moves:
        legal_moves = game.legal_moves()
        assert len(legal_moves) > 0

        # Pick a random move
        move = random.choice(legal_moves)
        success = game.make_move(move)
        assert success

        moves_made += 1

    # Game should be over (either win or draw)
    assert game.is_over()
    outcome = game.outcome()
    assert outcome is not None


def test_outcome_properties() -> None:
    # Test Red win
    game = spooky_connect4.Game(width=7, height=6)

    # Create a vertical win for Red
    for i in range(3):
        game.make_move(spooky_connect4.Move(0, i))
        game.make_move(spooky_connect4.Move(1, i))

    game.make_move(spooky_connect4.Move(0, 3))

    assert game.is_over()
    outcome = game.outcome()
    assert outcome is not None
    assert outcome.winner() == spooky_connect4.RED
    assert not outcome.is_draw()
    assert "Red" in outcome.name()


def test_board_representation() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Make some moves
    game.make_move(spooky_connect4.Move(0, 0))
    game.make_move(spooky_connect4.Move(1, 0))

    board_str = str(game.board())

    # Should contain board elements
    assert "|" in board_str
    assert "R" in board_str  # Red piece
    assert "Y" in board_str  # Yellow piece
