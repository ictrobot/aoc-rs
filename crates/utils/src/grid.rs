//! Grid helpers.

use crate::input::InputError;

/// Parse 2D grid.
///
/// This function assumes that one byte represents each item in the grid.
///
/// Returns (number of rows, number of columns, data) on success.
///
/// # Examples
///
/// ```
/// # use utils::grid::from_str;
/// assert_eq!(
///     from_str("##.#\n#..#\n#.##", |c| match c {
///         b'#' => Some(true),
///         b'.' => Some(false),
///         _ => None,
///     }).unwrap(),
///     (3, 4, vec![
///         true, true, false, true,
///         true, false, false, true,
///         true, false, true, true,
///     ]),
/// );
/// ```
pub fn from_str<T>(
    input: &str,
    mut func: impl FnMut(u8) -> Option<T>,
) -> Result<(usize, usize, Vec<T>), InputError> {
    let mut data = Vec::with_capacity(input.len());
    let mut lines = input.lines().peekable();

    let Some(&first_line) = lines.peek() else {
        return Err(InputError::new(input, input, "expected grid"));
    };

    let columns = first_line.len().max(1);
    for line in lines {
        if line.len() != columns {
            return Err(InputError::new(
                input,
                line,
                format!("expected {columns} column(s)"),
            ));
        }

        for b in line.bytes() {
            if let Some(v) = func(b) {
                data.push(v);
            } else {
                return Err(InputError::new(input, b as char, "invalid character"));
            }
        }
    }

    let rows = data.len() / columns;
    debug_assert_eq!(rows * columns, data.len());

    Ok((rows, columns, data))
}

/// Parse 2D grid, adding padding around the edges.
///
/// Similar to [`from_str`], but pads the edges of the parsed grid with `padding` rows and columns
/// filled with the default value. This is helpful to avoid bounds checks when considering a
/// location's neighbors.
///
/// Returns (number of rows, number of columns, data) on success (row and column counts include
/// the added padding).
///
/// # Examples
///
/// ```
/// # use utils::grid::from_str_padded;
/// assert_eq!(
///     from_str_padded("##.#\n#..#\n#.##", 2, false, |c| match c {
///         b'#' => Some(true),
///         b'.' => Some(false),
///         _ => None,
///     }).unwrap(),
///     (7, 8, vec![
///         false, false, false, false, false, false, false, false,
///         false, false, false, false, false, false, false, false,
///         false, false, true, true, false, true, false, false,
///         false, false, true, false, false, true, false, false,
///         false, false, true, false, true, true, false, false,
///         false, false, false, false, false, false, false, false,
///         false, false, false, false, false, false, false, false,
///     ]),
/// );
/// ```
pub fn from_str_padded<T: Clone>(
    input: &str,
    padding: usize,
    padding_value: T,
    mut func: impl FnMut(u8) -> Option<T>,
) -> Result<(usize, usize, Vec<T>), InputError> {
    let mut data = Vec::with_capacity(input.len());
    let mut lines = input.lines().peekable();

    let Some(&first_line) = lines.peek() else {
        return Err(InputError::new(input, input, "expected grid"));
    };

    let columns = first_line.len().max(1);
    let padded_columns = columns + 2 * padding;

    // Add initial padding rows + padding for start of first actual row
    data.resize(padded_columns * padding + padding, padding_value.clone());

    for line in lines {
        if line.len() != columns {
            return Err(InputError::new(
                input,
                line,
                format!("expected {columns} column(s)"),
            ));
        }

        for b in line.bytes() {
            if let Some(v) = func(b) {
                data.push(v);
            } else {
                return Err(InputError::new(input, b as char, "invalid character"));
            }
        }

        // Add padding for the end of the current row, and the start of the next row
        data.resize(data.len() + 2 * padding, padding_value.clone());
    }

    // Add final padding rows, minus the already added padding for the start of a row
    data.resize(
        data.len() + padded_columns * padding - padding,
        padding_value,
    );

    let rows = data.len() / padded_columns;
    debug_assert_eq!(rows * padded_columns, data.len());

    Ok((rows, padded_columns, data))
}

/// Checks that the provided grid has walls on each edge.
///
/// # Examples
/// ```
/// # use utils::grid::is_enclosed;
/// assert_eq!(
///     is_enclosed(5, 6, &[
///         b'#', b'#', b'#', b'#', b'#', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'#',
///         b'#', b'#', b'#', b'#', b'#', b'#',
///     ], |&b| b == b'#'),
///     true,
/// );
/// assert_eq!(
///     is_enclosed(5, 6, &[
///         b'#', b'#', b'#', b'#', b'#', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'#',
///         b'#', b'.', b'.', b'.', b'.', b'.',
///         b'#', b'#', b'#', b'#', b'#', b'#',
///     ], |&b| b == b'#'),
///     false,
/// );
/// ```
pub fn is_enclosed<T>(rows: usize, cols: usize, grid: &[T], is_wall: impl Fn(&T) -> bool) -> bool {
    grid[..cols].iter().all(&is_wall)
        && grid[(rows - 1) * cols..].iter().all(&is_wall)
        && (1..rows).all(|r| is_wall(&grid[r * cols]) && is_wall(&grid[(r + 1) * cols - 1]))
}
