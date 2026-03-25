pub const MIN_BOARD_DIMENSION: u8 = 4;
pub const MAX_BOARD_DIMENSION: u8 = 32;

#[inline]
pub const fn board_dimension_is_valid(dimension: u8) -> bool {
    dimension >= MIN_BOARD_DIMENSION && dimension <= MAX_BOARD_DIMENSION
}

#[inline]
pub fn assert_valid_board_dimensions(width: u8, height: u8) {
    assert!(
        board_dimension_is_valid(width),
        "Board width must be between {} and {}",
        MIN_BOARD_DIMENSION,
        MAX_BOARD_DIMENSION
    );
    assert!(
        board_dimension_is_valid(height),
        "Board height must be between {} and {}",
        MIN_BOARD_DIMENSION,
        MAX_BOARD_DIMENSION
    );
}
