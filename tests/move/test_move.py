import spooky_connect4


def test_move_creation() -> None:
    move = spooky_connect4.Move(0, 0)
    assert move is not None


def test_move_col_row() -> None:
    move = spooky_connect4.Move(3, 2)
    assert move.col() == 3
    assert move.row() == 2


def test_move_encode() -> None:
    move = spooky_connect4.Move(5, 0)
    encoded = move.encode()
    assert encoded == 5  # Column number


def test_move_str() -> None:
    move = spooky_connect4.Move(2, 1)
    s = str(move)
    assert isinstance(s, str)
    assert "col" in s.lower()


def test_move_repr() -> None:
    move = spooky_connect4.Move(4, 3)
    r = repr(move)
    assert "Move" in r


def test_move_equality() -> None:
    move1 = spooky_connect4.Move(1, 2)
    move2 = spooky_connect4.Move(1, 2)
    move3 = spooky_connect4.Move(2, 1)

    assert move1 == move2
    assert move1 != move3


def test_move_hash() -> None:
    move1 = spooky_connect4.Move(0, 0)
    move2 = spooky_connect4.Move(0, 0)
    move3 = spooky_connect4.Move(1, 1)

    # Same moves should have same hash
    assert hash(move1) == hash(move2)
    # Different moves should (usually) have different hashes
    assert hash(move1) != hash(move3)


def test_move_in_set() -> None:
    move1 = spooky_connect4.Move(0, 0)
    move2 = spooky_connect4.Move(0, 0)
    move3 = spooky_connect4.Move(1, 1)

    move_set = {move1, move3}
    assert move2 in move_set
    assert move3 in move_set
