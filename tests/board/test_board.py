import spooky_connect4


def test_board_empty() -> None:
    board = spooky_connect4.Board(width=7, height=6)
    assert not board.is_board_full()


def test_board_get_empty() -> None:
    board = spooky_connect4.Board(width=7, height=6)
    for col in range(7):
        for row in range(6):
            assert board.get_piece(col=col, row=row) is None


def test_board_column_height_empty() -> None:
    board = spooky_connect4.Board(width=7, height=6)
    for col in range(7):
        assert board.column_height(col) == 0


def test_board_column_full() -> None:
    board = spooky_connect4.Board(width=7, height=6)
    for col in range(7):
        assert not board.is_column_full(col)
