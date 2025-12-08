//! Disjoint-set data structures.

/// Disjoint-set union (DSU) data structure.
///
/// # Example
/// ```
/// # use utils::disjoint_set::Dsu;
/// let mut dsu = Dsu::new(5);
///
/// for i in 0..5 {
///     assert_eq!(dsu.find(i), i);
///     assert_eq!(dsu.set_size(i), 1);
/// }
///
/// assert!(dsu.union(0, 1));
/// assert!(dsu.union(1, 2));
/// assert!(!dsu.union(0, 2));
/// assert!(dsu.union(3, 4));
///
/// assert_eq!(dsu.find(0), dsu.find(1));
/// assert_eq!(dsu.find(0), dsu.find(2));
/// assert_eq!(dsu.set_size(0), 3);
///
/// assert_eq!(dsu.find(3), dsu.find(4));
/// assert_eq!(dsu.set_size(3), 2);
/// assert_ne!(dsu.find(0), dsu.find(3));
///
/// assert!(dsu.union(1, 3));
/// assert_eq!(dsu.set_size(0), 5);
/// for i in 1..5 {
///     assert_eq!(dsu.find(0), dsu.find(i));
/// }
/// ```
#[derive(Debug)]
pub struct Dsu {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl Dsu {
    /// Creates a new disjoint-set union data structure with `n` elements.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    /// Find the root of the set containing the element at index `x`.
    ///
    /// This requires `&mut self` because it performs path compression, making later calls faster.
    #[inline]
    #[must_use]
    pub fn find(&mut self, mut x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        while self.parent[x] != root {
            (x, self.parent[x]) = (self.parent[x], root);
        }
        root
    }

    /// Merges the sets containing index `a` and `b`.
    ///
    /// Returns `true` if the sets were merged, `false` if they were already in the same set.
    #[inline]
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let mut ra = self.find(a);
        let mut rb = self.find(b);
        if ra == rb {
            return false;
        }
        if self.size[ra] < self.size[rb] {
            (ra, rb) = (rb, ra);
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
        true
    }

    /// Returns the size of the set containing the element at index `x`.
    ///
    /// This requires `&mut self` because it performs path compression, making later calls faster.
    #[inline]
    #[must_use]
    pub fn set_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        self.size[root]
    }

    /// Returns the size of the set with root `x`.
    ///
    /// This function will panic if `x` is not a root.
    #[inline]
    #[must_use]
    pub fn root_size(&self, x: usize) -> usize {
        assert_eq!(self.parent[x], x);
        self.size[x]
    }

    /// Returns an iterator over the roots of the disjoint-set.
    #[inline]
    pub fn roots(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.parent.len()).filter(|&x| self.parent[x] == x)
    }
}
