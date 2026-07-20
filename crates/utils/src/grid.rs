//! Grid helpers.

use crate::input::InputError;
use std::error::Error;
use std::hint::cold_path;

/// A parsed grid: `(number of rows, number of columns, data)`.
pub type Grid<T> = (usize, usize, Vec<T>);

/// Parse a 2D grid.
///
/// This function assumes that each byte represents one item in the grid.
/// Using 1 byte wide output types is recommended to enable more efficient vectorization.
///
/// Parsing is done in two passes per line:
///
/// 1. A "hot" pass, where each byte is mapped to an output value and checked for validity.
///    This uses the `hot_map` and `hot_valid` functions, which should be pure and total mappings
///    from [`u8`] to their respective outputs to enable vectorization.
///    Additionally, `hot_valid` must return false for both `'\r'` and `'\n'`.
///
/// 2. A "slow" pass, where any invalid bytes from the first pass are re-processed with their index
///    in the final grid.
///    This uses the `slow_map` function which returns a [`Result`], containing either a mapped
///    value or an error.
///    `slow_map` can be used to perform more complex, non-pure, mappings such as storing positions
///    and will never be called for newlines or bytes that were valid in the first pass.
///    If all the bytes were valid in the first pass, this pass is skipped.
///
/// `default_value` is used to initialize the grid before parsing and for any padding.
/// A value with an all-zero bit pattern will usually faster to initialize and is recommended when
/// padding is not used.
///
/// Returns (number of rows, number of columns, data) on success.
///
/// # Examples
///
/// ```
/// # use utils::grid;
/// assert_eq!(
///     grid::parse(
///         /* input         */ "##.#\n#..#\n#.##",
///         /* padding       */ 0,
///         /* default_value */ false,
///         /* hot_map       */ |b| b == b'#',
///         /* hot_valid     */ |b| matches!(b, b'.' | b'#'),
///         /* slow_map      */ |_, _| Err("expected '.' or '#'"),
///     ).unwrap(),
///     (3, 4, vec![
///         true, true, false, true,
///         true, false, false, true,
///         true, false, true, true,
///     ]),
/// );
/// ```
///
///
/// ```
/// # use utils::grid;
/// assert_eq!(
///     grid::parse(
///         /* input         */"##.#\n#..#\n#.##",
///         /* padding       */ 2,
///         /* default_value */ false,
///         /* hot_map       */ |b| b == b'#',
///         /* hot_valid     */ |b| matches!(b, b'.' | b'#'),
///         /* slow_map      */ |_, _| Err("expected '.' or '#'"),
///     ).unwrap(),
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
///
/// ```
/// # use utils::grid;
/// let mut start = None;
/// assert_eq!(
///     grid::parse(
///         /* input         */ ".0.#S\r\n..1..\r\n.###2\r\n.3...",
///         /* padding       */ 1,
///         /* default_value */ b'#',
///         /* hot_map       */ |b| b,
///         /* hot_valid     */ |b| matches!(b, b'.' | b'#' | b'0'..=b'9'),
///         /* slow_map      */ |i, b| {
///             match b {
///                 b'S' if start.is_none() => {
///                     start = Some(i);
///                     Ok(b'.')
///                 },
///                 b'S' => Err("expected only one 'S'"),
///                 _ => Err("expected '.', '#', 'S' or a digit")
///             }
///         },
///     ).unwrap(),
///     (6, 7, vec![
///         b'#', b'#', b'#', b'#', b'#', b'#', b'#',
///         b'#', b'.', b'0', b'.', b'#', b'.', b'#',
///         b'#', b'.', b'.', b'1', b'.', b'.', b'#',
///         b'#', b'.', b'#', b'#', b'#', b'2', b'#',
///         b'#', b'.', b'3', b'.', b'.', b'.', b'#',
///         b'#', b'#', b'#', b'#', b'#', b'#', b'#',
///     ]),
/// );
/// assert_eq!(start, Some(12));
/// ```
#[inline]
pub fn parse<T: Clone, E: Into<Box<dyn Error>>>(
    input: &str,
    padding: usize,
    default_value: T,
    hot_map: impl Fn(u8) -> T,
    hot_valid: impl Fn(u8) -> bool,
    mut slow_map: impl FnMut(usize, u8) -> Result<T, E>,
) -> Result<Grid<T>, InputError> {
    assert!(!hot_valid(b'\n'));
    assert!(!hot_valid(b'\r'));
    let g = parse_grid_shape(input)?;
    let padded_cols = g.cols + 2 * padding;
    let padded_rows = g.rows + 2 * padding;

    let mut data = vec![default_value; padded_cols * padded_rows];
    for ((r, input_line), data_line) in input
        .as_bytes()
        .chunks(g.line_length)
        .enumerate()
        .zip(data.chunks_exact_mut(padded_cols).skip(padding))
    {
        // Hot pass
        let mut valid = true;
        for (&b, d) in input_line[..g.cols]
            .iter()
            .zip(data_line[padding..].iter_mut())
        {
            *d = hot_map(b);
            valid &= hot_valid(b);
        }

        // Slow pass
        if !valid {
            for ((c, &b), d) in input_line[..g.cols]
                .iter()
                .enumerate()
                .zip(data_line[padding..].iter_mut())
            {
                if hot_valid(b) {
                    continue;
                }
                if b == b'\n' || b == b'\r' {
                    return Err(InputError::new(
                        input,
                        &input_line[c..],
                        format!("expected {} columns", g.cols),
                    ));
                }

                let index = (r + padding) * padded_cols + padding + c;
                match slow_map(index, b) {
                    Ok(v) => *d = v,
                    Err(err) => return Err(InputError::new(input, &input_line[c..], err)),
                }
            }
        }

        // Check input_line ends in the expected newline.
        // Do this after the slow pass, so any earlier newlines earlier in the chunk have already
        // been caught.
        check_newline(input, &g, r, input_line)?;
    }

    Ok((padded_rows, padded_cols, data))
}

#[derive(Clone, Copy)]
struct GridShape {
    cols: usize,
    rows: usize,
    line_length: usize,
    crlf_endings: bool,
}

#[inline]
fn parse_grid_shape(input: &str) -> Result<GridShape, InputError> {
    let Some(line_length) = input
        .bytes()
        .position(|b| b == b'\n')
        .map(|x| x + 1)
        .filter(|&x| x >= 2)
    else {
        return Err(InputError::new(input, 0, "expected grid"));
    };

    let crlf_endings = input.as_bytes()[line_length - 2] == b'\r';
    let newline_length = 1 + usize::from(crlf_endings);
    if !(input.len() + newline_length).is_multiple_of(line_length) {
        return Err(InputError::new(
            input,
            input.len(),
            "expected input length to be a multiple of the first line length",
        ));
    }

    Ok(GridShape {
        cols: line_length - newline_length,
        rows: (input.len() + newline_length) / line_length,
        line_length,
        crlf_endings,
    })
}

#[inline]
fn check_newline(
    input: &str,
    parsed_grid: &GridShape,
    row: usize,
    row_input: &[u8],
) -> Result<(), InputError> {
    if row != parsed_grid.rows - 1
        && if parsed_grid.crlf_endings {
            row_input.last_chunk() != Some(b"\r\n")
        } else {
            row_input.last_chunk() != Some(b"\n")
        }
    {
        cold_path();
        return Err(InputError::new(
            input,
            &row_input[row_input.len() - 2..],
            "expected newline",
        ));
    }
    Ok(())
}

/// Iterate over each row of a 2D grid and pass it to a callback after validation.
///
/// Unlike [`parse`], this does not support padding or allocate a transformed grid. The callback
/// receives the row index, column count and row input bytes. The callback may be called with one
/// or more rows before an error is found in a later row.
///
/// # Examples
///
/// ```
/// # use utils::{grid, input::InputError};
/// let mut row_masks = Vec::new();
/// let input = "#.#.#\n.#.#.\n#####";
/// let shape = grid::for_each_row(
///     input,
///     |b| matches!(b, b'.' | b'#'),
///     || "expected '.' or '#'",
///     |row, cols, row_bytes| {
///         if row == 0 && cols > 32 {
///             return Err(InputError::new(input, 0, "expected at most 32 columns"));
///         }
///         let mut bits = 0u32;
///         for (col, &b) in row_bytes.iter().enumerate() {
///             bits |= u32::from(b == b'#') << col;
///         }
///         row_masks.push(bits);
///         Ok(())
///     },
/// ).unwrap();
/// assert_eq!(shape, (3, 5));
/// assert_eq!(row_masks, vec![0b10101, 0b01010, 0b11111]);
/// ```
#[inline]
pub fn for_each_row<E: Into<Box<dyn Error>>>(
    input: &str,
    hot_valid: impl Fn(u8) -> bool,
    invalid_error: impl Fn() -> E,
    mut callback: impl FnMut(usize, usize, &[u8]) -> Result<(), InputError>,
) -> Result<(usize, usize), InputError> {
    assert!(!hot_valid(b'\n'));
    assert!(!hot_valid(b'\r'));
    let g = parse_grid_shape(input)?;

    for (r, input_line) in input.as_bytes().chunks(g.line_length).enumerate() {
        let mut all_valid = true;
        for &b in &input_line[..g.cols] {
            all_valid &= hot_valid(b);
        }

        if !all_valid {
            cold_path();
            for (c, &b) in input_line[..g.cols].iter().enumerate() {
                if !hot_valid(b) {
                    return Err(InputError::new(input, &input_line[c..], invalid_error()));
                }
            }
            unreachable!();
        }

        check_newline(input, &g, r, input_line)?;

        callback(r, g.cols, &input_line[..g.cols])?;
    }

    Ok((g.rows, g.cols))
}

/// Parse a "standard" maze with open tiles `.`, walls `#` and one start `S` and one end `E`.
///
/// Returns ((number of rows, number of columns, data), start index, end index) on success.
///
/// # Examples
/// ```
/// # use utils::grid;
/// assert_eq!(
///     grid::parse_maze("...#S\n.#E#.\n.###.\n.....", 1).unwrap(),
///     (
///         (6, 7, vec![
///             b'#', b'#', b'#', b'#', b'#', b'#', b'#',
///             b'#', b'.', b'.', b'.', b'#', b'.', b'#',
///             b'#', b'.', b'#', b'.', b'#', b'.', b'#',
///             b'#', b'.', b'#', b'#', b'#', b'.', b'#',
///             b'#', b'.', b'.', b'.', b'.', b'.', b'#',
///             b'#', b'#', b'#', b'#', b'#', b'#', b'#',
///         ]),
///         12,
///         17,
///     )
/// );
/// ```
#[inline]
pub fn parse_maze(input: &str, padding: usize) -> Result<(Grid<u8>, usize, usize), InputError> {
    let mut start = None;
    let mut end = None;
    let grid = parse(
        input,
        padding,
        if padding > 0 { b'#' } else { 0 },
        |b| b,
        |b| matches!(b, b'.' | b'#'),
        |i, b| {
            match b {
                b'S' if start.is_none() => start = Some(i),
                b'S' => return Err("expected one 'S'"),
                b'E' if end.is_none() => end = Some(i),
                b'E' => return Err("expected one 'E'"),
                _ => return Err("expected '.', '#', 'S' or 'E'"),
            }
            Ok(b'.')
        },
    )?;
    let Some(start) = start else {
        return Err(InputError::new(input, 0, "expected one 'S'"));
    };
    let Some(end) = end else {
        return Err(InputError::new(input, 0, "expected one 'E'"));
    };
    Ok((grid, start, end))
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
