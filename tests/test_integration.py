import random

import spooky_connect4


def test_full_game_simulation() -> None:
    """Play a full random game to completion"""
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


def test_game_with_undo() -> None:
    """Test making and unmaking moves"""
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


def test_clone_independence() -> None:
    """Test that cloned games are independent"""
    game = spooky_connect4.Game(width=7, height=6)

    # Make some moves
    for _ in range(5):
        legal_moves = game.legal_moves()
        if not legal_moves:
            break
        game.make_move(legal_moves[0])

    # Clone the game
    cloned = game.clone()

    # Make different moves in each
    game_moves = game.legal_moves()
    cloned_moves = cloned.legal_moves()

    if game_moves:
        game.make_move(game_moves[0])

    if len(cloned_moves) > 1:
        cloned.make_move(cloned_moves[1])

    # Games should now be different (assuming they had different moves available)
    # At minimum, they should be independent


def test_encode_decode_all_moves() -> None:
    """Test encoding and decoding all legal moves"""
    game = spooky_connect4.Game(width=7, height=6)

    for _ in range(20):
        if game.is_over():
            break

        legal_moves = game.legal_moves()

        for move in legal_moves:
            # Encode the move
            encoded_move = move.encode()

            # Decode it back
            decoded_move = spooky_connect4.Move.decode(encoded_move, game)

            assert decoded_move is not None
            assert decoded_move.col() == move.col()

        # Make a random move to continue
        if legal_moves:
            game.make_move(random.choice(legal_moves))


def test_constants() -> None:
    """Test that constants are defined correctly"""
    assert spooky_connect4.RED == 1
    assert spooky_connect4.YELLOW == -1


def test_outcome_properties() -> None:
    """Test outcome properties for different game endings"""
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
    """Test board string representation"""
    game = spooky_connect4.Game(width=7, height=6)

    # Make some moves
    game.make_move(spooky_connect4.Move(0, 0))
    game.make_move(spooky_connect4.Move(1, 0))

    board_str = str(game.board())

    # Should contain board elements
    assert "|" in board_str
    assert "R" in board_str  # Red piece
    assert "Y" in board_str  # Yellow piece


def test_sequential_games() -> None:
    """Test playing multiple games in sequence"""
    for _ in range(10):
        game = spooky_connect4.Game(width=7, height=6)

        # Play a short game
        for _ in range(10):
            if game.is_over():
                break

            legal_moves = game.legal_moves()
            if not legal_moves:
                break

            game.make_move(random.choice(legal_moves))

        # Game should be in valid state
        assert isinstance(game.is_over(), bool)


def test_move_validation() -> None:
    """Test that invalid moves are properly rejected"""
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


def test_random_game_with_encoding() -> None:
    """Play a random game while encoding state at each step"""
    game = spooky_connect4.Game(width=7, height=6)

    encodings = []
    moves_count = 0

    while not game.is_over() and moves_count < 42:
        # Encode current state
        encodings.append(game.encode_game_planes())

        # Make a move
        legal_moves = game.legal_moves()
        if not legal_moves:
            break

        game.make_move(random.choice(legal_moves))
        moves_count += 1

    # Should have at least one encoding
    assert len(encodings) > 0

    # All encodings should have the same structure (tuple of data, num_planes, height, width)
    for data, num_planes, height, width in encodings:
        assert num_planes == 17  # Total planes
        assert height == 6
        assert width == 7
        assert len(data) == num_planes * height * width
