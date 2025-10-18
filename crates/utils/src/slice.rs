//! Slice helpers.
use std::cmp::Ordering;

/// Merges a sorted deduped [`Vec`] in place with a sorted deduped slice.
///
/// This is equivalent to a set union on sorted data.
///
/// Both inputs **must** be sorted and contain no duplicates.
///
/// # Examples
/// ```
/// # use utils::slice::merge_sorted_deduped_in_place;
/// let mut v = vec![1, 3, 5];
/// merge_sorted_deduped_in_place(&mut v, &[2, 3, 4]);
/// assert_eq!(v, [1, 2, 3, 4, 5]);
///
/// // Elements at start and end
/// let mut v = vec![1, 2, 3, 4, 5];
/// merge_sorted_deduped_in_place(&mut v, &[0, 6]);
/// assert_eq!(v, [0, 1, 2, 3, 4, 5, 6]);
///
/// // Complete overlap
/// let mut v = vec![1, 2, 3];
/// merge_sorted_deduped_in_place(&mut v, &[1, 2, 3]);
/// assert_eq!(v, [1, 2, 3]);
///
/// // No overlap
/// let mut v = vec![1, 2];
/// merge_sorted_deduped_in_place(&mut v, &[3, 4]);
/// assert_eq!(v, [1, 2, 3, 4]);
///
/// let mut v = vec![24, 53];
/// merge_sorted_deduped_in_place(&mut v, &[6]);
/// assert_eq!(v, [6, 24, 53]);
///
/// // Partial overlap
/// let mut v = vec![10, 11, 12, 13];
/// merge_sorted_deduped_in_place(&mut v, &[13, 14]);
/// assert_eq!(v, [10, 11, 12, 13, 14]);
///
/// let mut v = vec![300, 400, 500];
/// merge_sorted_deduped_in_place(&mut v, &[100, 200, 300]);
/// assert_eq!(v, [100, 200, 300, 400, 500]);
///
/// // Empty inputs
/// let mut v = Vec::new();
/// merge_sorted_deduped_in_place(&mut v, &[5]);
/// assert_eq!(v, [5]);
///
/// let mut v = vec![6];
/// merge_sorted_deduped_in_place(&mut v, &[]);
/// assert_eq!(v, [6]);
///
/// let mut v: Vec<u32> = Vec::new();
/// merge_sorted_deduped_in_place(&mut v, &[]);
/// assert_eq!(v, []);
/// ```
#[inline]
pub fn merge_sorted_deduped_in_place<T: Copy + Ord + Default>(a: &mut Vec<T>, b: &[T]) {
    debug_assert!(a.windows(2).all(|w| w[0] < w[1]));
    debug_assert!(b.windows(2).all(|w| w[0] < w[1]));

    let mut new = 0;
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            Ordering::Less => i += 1,
            Ordering::Equal => {
                i += 1;
                j += 1;
            }
            Ordering::Greater => {
                new += 1;
                j += 1;
            }
        }
    }
    new += b.len() - j;

    if new == 0 {
        return;
    }

    let (mut i, mut j, mut write) = (a.len(), b.len(), a.len() + new);
    a.resize(a.len() + new, T::default());

    while i > 0 && j > 0 {
        write -= 1;
        a[write] = match a[i - 1].cmp(&b[j - 1]) {
            Ordering::Less => {
                j -= 1;
                b[j]
            }
            Ordering::Equal => {
                i -= 1;
                j -= 1;
                a[i]
            }
            Ordering::Greater => {
                i -= 1;
                a[i]
            }
        }
    }

    a[..j].copy_from_slice(&b[..j]);
}
