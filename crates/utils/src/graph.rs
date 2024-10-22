//! Graph helpers.

use crate::bit::BitIterator;
use crate::number::UnsignedInteger;
use std::marker::PhantomData;

/// Explore all hamiltonian paths/cycles in a graph.
///
/// # Panics
/// This function panics if the number of vertices exceeds the maximum supported, currently 32.
///
/// # Examples
///
/// Shortest hamiltonian path and cycle starting at vertex 0:
/// ```
/// # use utils::graph::explore_hamiltonian_paths;
/// # use std::ops::Add;
/// let distance_matrix = vec![
///     0, 7, 2, 8, 6,
///     7, 0, 1, 3, 5,
///     2, 1, 0, 9, 4,
///     8, 3, 9, 0, 9,
///     6, 5, 4, 9, 0,
/// ];
///
/// let mut min_hamiltonian_path = u32::MAX;
/// let mut min_hamiltonian_cycle = u32::MAX;
/// explore_hamiltonian_paths(
///     5,                                                              // Number of vertices
///     0,                                                              // Start vertex
///     0,                                                              // Initial path cost
///     |a, b| distance_matrix[a as usize * 5 + b as usize],            // Distance function
///     u32::add,                                                       // Accumulate function
///     |path, loop_edge| {                                             // Callback function
///         min_hamiltonian_path = min_hamiltonian_path.min(path);
///         min_hamiltonian_cycle = min_hamiltonian_cycle.min(path + loop_edge);
///     }
/// );
///
/// assert_eq!(min_hamiltonian_path, 14);  // 0 =[6]=> 4 =[4]=> 2 =[1]=> 1 =[2]=> 3
/// assert_eq!(min_hamiltonian_cycle, 21); // 0 =[2]=> 2 =[1]=> 1 =[3]=> 3 =[9]=> 4 =[6]=> 0
/// ```
///
/// Shortest and longest hamilton paths starting at any vertex:
/// ```
/// # use utils::graph::explore_hamiltonian_paths;
/// let distance_matrix = vec![
///     0, 7, 2, 8, 6,
///     7, 0, 1, 3, 5,
///     2, 1, 0, 9, 4,
///     8, 3, 9, 0, 9,
///     6, 5, 4, 9, 0,
/// ];
///
/// let mut min_path = u32::MAX;
/// let mut max_path = 0;
/// explore_hamiltonian_paths(
///     5,                                                              // Number of vertices
///     0,                                                              // Start vertex
///     (0, u32::MAX, 0),                                               // Initial path cost
///     |a, b| distance_matrix[a as usize * 5 + b as usize],            // Distance function
///     |(total, min_edge, max_edge), edge| {                           // Accumulate function
///         (total + edge, min_edge.min(edge), max_edge.max(edge))
///     },
///     |(total, min_edge, max_edge), loop_edge| {                      // Callback function
///         let loop_total = total + loop_edge;
///         min_path = min_path.min(loop_total - max_edge.max(loop_edge));
///         max_path = max_path.max(loop_total - min_edge.min(loop_edge));
///     }
/// );
///
/// assert_eq!(min_path, 12); // 0 =[6]=> 4 =[4]=> 2 =[1]=> 1 =[2]=> 3
/// assert_eq!(max_path, 31); // 1 =[7]=> 0 =[6]=> 4 =[9]=> 3 =[9]=> 2
/// ```
#[inline]
pub fn explore_hamiltonian_paths<E, P: Copy>(
    vertices: u32,
    start_vertex: u32,
    initial_path: P,
    distance_fn: impl Fn(u32, u32) -> E,
    accumulate_fn: impl Fn(P, E) -> P,
    callback_fn: impl FnMut(P, E),
) {
    // Rust doesn't allow recursive FnMut closures, so move state into struct
    struct Visitor<P, F, G, H> {
        start_vertex: u32,
        phantom: PhantomData<P>,
        distance_fn: F,
        accumulate_fn: G,
        callback_fn: H,
    }

    impl<E, P: Copy, F: Fn(u32, u32) -> E, G: Fn(P, E) -> P, H: FnMut(P, E)> Visitor<P, F, G, H> {
        #[inline]
        fn visit<V: UnsignedInteger>(&mut self, current_vertex: u32, visited: V, path: P) {
            if visited == V::MAX {
                let loop_edge = (self.distance_fn)(current_vertex, self.start_vertex);
                (self.callback_fn)(path, loop_edge);
                return;
            }

            for (next_vertex, next_bit) in BitIterator::zeroes(visited) {
                let new_edge = (self.distance_fn)(current_vertex, next_vertex);
                let new_path = (self.accumulate_fn)(path, new_edge);
                self.visit(next_vertex, visited | next_bit, new_path);
            }
        }
    }

    // visit can take any unsigned integer type as a bitmask, so the code could switch the
    // implementation based on the number of vertices. However, it seems unlikely that an AoC
    // problem would require this as generating that many permutations would be very slow, so for
    // now assume u32 is fine.
    assert!(vertices <= 32, "too many vertices");

    let mut visitor = Visitor {
        start_vertex,
        distance_fn,
        accumulate_fn,
        callback_fn,
        phantom: PhantomData,
    };
    visitor.visit(
        start_vertex,
        !(u32::MAX >> (u32::BITS - vertices)) | (1 << start_vertex),
        initial_path,
    );
}
