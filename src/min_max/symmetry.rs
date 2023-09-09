use std::collections::{HashMap, HashSet};
use std::mem;
use enumset::{EnumSetType, EnumSet};
use itertools::Itertools;
use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;


pub trait Symmetry<T> {
    fn canonicalize(&self, target: &T) -> T;
    fn expand(&self, normalised: &T) -> Vec<T>;
}

#[derive(EnumIter, EnumSetType, Debug)]
#[enumset(repr = "u8")]
pub enum GridSymmetryAxis {
    Vertical,
    Horizontal,
    DiagonalTopBottom,
    DiagonalBottomTop,
}

impl GridSymmetryAxis {
    pub fn symmetric_indices_3x3(&self) -> [(usize, usize); 3] {
        match self {
            GridSymmetryAxis::Vertical => [(0, 2), (3, 5), (6, 8)],
            GridSymmetryAxis::Horizontal => [(0, 6), (1, 7), (2, 8)],
            GridSymmetryAxis::DiagonalTopBottom => [(1, 3), (2, 6), (5, 7)],
            GridSymmetryAxis::DiagonalBottomTop => [(0, 8), (1, 5), (3, 7)],
        }
    }

    fn expand_index(&self, index: usize) -> Vec<usize> {
        let mut result = vec![index];
        result.extend(self.symmetric_indices_3x3().iter().filter_map(|(first, second)| {
            if index == *first {
                Some(*second)
            } else if index == *second {
                Some(*first)
            } else {
                None
            }
        }));
        return result;
    }
}

pub type GridSymmetryAxes = EnumSet<GridSymmetryAxis>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct GridSymmetry3x3 {
    axes: GridSymmetryAxes,
    canonical_index: &'static Vec<usize>,
}

impl GridSymmetry3x3 {
    pub fn new<A>(axes: A) -> Self where A: Into<GridSymmetryAxes> {
        let set = axes.into();
        let canonical_index = AXIS_TO_CANONICAL_INDICES_3X3.get(&set).expect("map contains all combinations ");
        Self { axes: set, canonical_index }
    }
}

lazy_static! {
    // basically a two staged mapping
    // symmetry axis -> index -> canonical index
    static ref AXIS_TO_CANONICAL_INDICES_3X3: HashMap<GridSymmetryAxes, Vec<usize>> = GridSymmetryAxis::iter().powerset().map(|axis| {
        let axis_set = axis.into_iter().collect::<EnumSet<_>>();
        // for each board index compute the canonical index (considering the symmetries)
        let canonical_indices = (0..=8).into_iter().map(|original_index| {
            // fixpoint iteration to find all indices that are equivalent to the given one
            let mut current = HashSet::new();
            current.insert(original_index);
            let mut old = current.clone();

            loop {
                for index in &old {
                    current.extend(axis_set.iter().flat_map(|axis| axis.expand_index(*index)))
                }

                if current == old {
                    break;
                }
                mem::swap(&mut current, &mut old);
            }
            // the canonical index is the smallest
            return *current.iter().min().expect("set always contains original_index");
        }).collect();

        return (axis_set, canonical_indices);
    }).collect();
}


impl Symmetry<usize> for GridSymmetry3x3 {
    fn canonicalize(&self, target: &usize) -> usize {
        debug_assert!(*target <= 8);
        self.canonical_index[*target]
    }

    fn expand(&self, normalised: &usize) -> Vec<usize> {
        self.axes.iter().flat_map(|axis| axis.expand_index(*normalised)).collect()
    }
}

#[cfg(test)]
mod test {
    use enumset::EnumSet;
    use crate::min_max::symmetry::{GridSymmetry3x3, GridSymmetryAxis, Symmetry};

    #[test]
    fn grid_symmetry3x3_normalize() {
        let vertical = GridSymmetry3x3::new(GridSymmetryAxis::Vertical);
        assert_eq!(vertical.canonicalize(&0usize), 0);
        assert_eq!(vertical.canonicalize(&1usize), 1);
        assert_eq!(vertical.canonicalize(&2usize), 0);
        assert_eq!(vertical.canonicalize(&4usize), 4);
        assert_eq!(vertical.canonicalize(&8usize), 6);

        let diagonal_horizontal = GridSymmetry3x3::new(GridSymmetryAxis::DiagonalTopBottom | GridSymmetryAxis::Horizontal);
        assert_eq!(diagonal_horizontal.canonicalize(&0usize), 0);
        assert_eq!(diagonal_horizontal.canonicalize(&1usize), 1);
        assert_eq!(diagonal_horizontal.canonicalize(&2usize), 0);
        assert_eq!(diagonal_horizontal.canonicalize(&4usize), 4);
        assert_eq!(diagonal_horizontal.canonicalize(&5usize), 1);
        assert_eq!(diagonal_horizontal.canonicalize(&8usize), 0);

        let all = GridSymmetry3x3::new(EnumSet::<GridSymmetryAxis>::ALL);
        assert_eq!(all.canonicalize(&0usize), 0);
        assert_eq!(all.canonicalize(&2usize), 0);
        assert_eq!(all.canonicalize(&8usize), 0);

        assert_eq!(all.canonicalize(&1usize), 1);
        assert_eq!(all.canonicalize(&3usize), 1);
        assert_eq!(all.canonicalize(&5usize), 1);

        assert_eq!(all.canonicalize(&4usize), 4);
    }

    #[test]
    fn grid_symmetry3x3_expand() {}
}