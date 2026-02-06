import spooky_connect4


def test_encode_state() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    data, num_planes, height, width = game.encode_game_planes()

    # Should return flat data with shape metadata
    assert isinstance(data, list)
    assert num_planes == 17
    assert height == 6
    assert width == 7
    assert len(data) == num_planes * height * width


def test_encode_state_length() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    data, num_planes, height, width = game.encode_game_planes()

    # Total planes = (HISTORY_LENGTH * PIECE_PLANES) + CONSTANT_PLANES
    # HISTORY_LENGTH = 8, PIECE_PLANES = 2, CONSTANT_PLANES = 1
    # Total = (8 * 2) + 1 = 17 planes
    assert num_planes == 17
    assert height == 6
    assert width == 7
    assert len(data) == 17 * 6 * 7


def test_encode_game_planes_count() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    data, num_planes, height, width = game.encode_game_planes()

    # Should have 17 planes total
    assert num_planes == 17


def get_plane_value(data: list[float], plane: int, row: int, col: int, height: int, width: int) -> float:
    return data[plane * height * width + row * width + col]


def test_encode_empty_game() -> None:
    game = spooky_connect4.Game(width=7, height=6)
    data, num_planes, height, width = game.encode_game_planes()

    # First two planes (current player and opponent) should be all zeros
    for plane in range(2):
        for row in range(height):
            for col in range(width):
                assert get_plane_value(data, plane, row, col, height, width) == 0.0


def test_encode_with_pieces() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Make a move
    move = spooky_connect4.Move(0, 0)
    game.make_move(move)

    data, num_planes, height, width = game.encode_game_planes()

    # Now should have a piece somewhere in the first two planes
    has_piece = False
    for plane in range(2):
        for row in range(height):
            for col in range(width):
                if get_plane_value(data, plane, row, col, height, width) != 0.0:
                    has_piece = True
                    break
    assert has_piece


def test_move_encode_decode() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    move = spooky_connect4.Move(3, 0)
    encoded = move.encode()
    assert encoded == 3

    decoded = spooky_connect4.Move.decode(encoded, game)
    assert decoded is not None
    assert decoded.col() == move.col()


def test_encode_different_players() -> None:
    game = spooky_connect4.Game(width=7, height=6)

    # Red's turn
    data_red, num_planes, height, width = game.encode_game_planes()

    # Make a move to switch to Yellow
    move = game.legal_moves()[0]
    game.make_move(move)

    # Yellow's turn
    data_yellow, _, _, _ = game.encode_game_planes()

    # Encodings should be different after a move
    assert data_red != data_yellow
