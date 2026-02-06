import random

import spooky_connect4


def test_random_game_with_encoding() -> None:
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


def test_encode_decode_all_moves() -> None:
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
