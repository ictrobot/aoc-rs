//! Grid helpers.

use crate::input::InputError;
use std::error::Error;

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

    // Line length including newline
    let Some(line_length) = input
        .bytes()
        .position(|b| b == b'\n')
        .map(|x| x + 1)
        .filter(|&x| x >= 2)
    else {
        return Err(InputError::new(input, 0, "expected grid"));
    };

    let clrf_endings = input.as_bytes()[line_length - 2] == b'\r';
    let newline_length = 1 + usize::from(clrf_endings);

    // Add line_ending_length to account for the final newline which should have already been
    // stripped by utils::input::strip_final_newline
    if !(input.len() + newline_length).is_multiple_of(line_length) {
        return Err(InputError::new(
            input,
            input.len(),
            "expected input length to be a multiple of the first line length",
        ));
    }

    let input_cols = line_length - newline_length;
    let input_rows = (input.len() + newline_length) / line_length;

    let padded_cols = input_cols + 2 * padding;
    let padded_rows = input_rows + 2 * padding;

    let mut data = vec![default_value; padded_cols * padded_rows];
    for ((r, input_line), data_line) in input
        .as_bytes()
        .chunks(line_length)
        .enumerate()
        .zip(data.chunks_exact_mut(padded_cols).skip(padding))
    {
        // Hot pass
        let mut valid = true;
        for (&b, d) in input_line[..input_cols]
            .iter()
            .zip(data_line[padding..].iter_mut())
        {
            *d = hot_map(b);
            valid &= hot_valid(b);
        }

        // Slow pass
        if !valid {
            for ((c, &b), d) in input_line[..input_cols]
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
                        format!("expected {input_cols} columns"),
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
        if r != input_rows - 1
            && if clrf_endings {
                input_line.last_chunk() != Some(b"\r\n")
            } else {
                input_line.last_chunk() != Some(b"\n")
            }
        {
            return Err(InputError::new(
                input,
                &input_line[input_line.len() - 2..],
                "expected newline",
            ));
        }
    }

    Ok((padded_rows, padded_cols, data))
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
