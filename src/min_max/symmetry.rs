use std::marker::PhantomData;
use std::mem;
use ahash::{HashMap, HashSet, HashSetExt};
use enumset::{EnumSetType, EnumSet};
use itertools::Itertools;
use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;


pub trait Symmetry<T> {
    fn canonicalize(&self, target: &T) -> T;
    fn expand(&self, normalised: &T) -> Vec<T>;
}


#[derive(Eq, PartialEq, Debug, Clone)]
pub struct SymmetricMove<I, S: Symmetry<I>>(pub I, pub S);

impl<T, S: Symmetry<T>> SymmetricMove<T, S> {
    pub fn index(&self) -> &T {
        &self.0
    }

    pub fn expanded_indices(&self) -> Vec<T> {
        self.1.expand(&self.0)
    }
}

#[derive(EnumIter, EnumSetType, Debug)]
#[enumset(repr = "u8")]
pub enum GridSymmetryAxis {
    Vertical,
    Horizontal,
    DiagonalTopBottom,
    DiagonalBottomTop,
}

pub type GridSymmetryAxes = EnumSet<GridSymmetryAxis>;

pub trait GridSymmetryAxisContext {
    type SymmetricPairs: IntoIterator<Item=(usize, usize)>;

    fn symmetric_indices(axis: GridSymmetryAxis) -> Self::SymmetricPairs;

    fn canonicalization_mapping(axes: &GridSymmetryAxes) -> &'static Vec<usize>;
    fn expand_index(axis: GridSymmetryAxis, index: usize) -> Vec<usize> {
        let mut result = vec![index];
        result.extend(Self::symmetric_indices(axis).into_iter().filter_map(|(first, second)| {
            if index == first {
                Some(second)
            } else if index == second {
                Some(first)
            } else {
                None
            }
        }));
        return result;
    }
}

#[derive(Debug)]
pub struct GridSymmetryAxisContext3x3;

impl GridSymmetryAxisContext for GridSymmetryAxisContext3x3 {
    type SymmetricPairs = [(usize, usize); 3];

    fn symmetric_indices(axis: GridSymmetryAxis) -> Self::SymmetricPairs {
        match axis {
            GridSymmetryAxis::Vertical => [(0, 2), (3, 5), (6, 8)],
            GridSymmetryAxis::Horizontal => [(0, 6), (1, 7), (2, 8)],
            GridSymmetryAxis::DiagonalTopBottom => [(1, 3), (2, 6), (5, 7)],
            GridSymmetryAxis::DiagonalBottomTop => [(0, 8), (1, 5), (3, 7)],
        }
    }
    fn canonicalization_mapping(axes: &GridSymmetryAxes) -> &'static Vec<usize> {
        AXIS_TO_CANONICAL_INDICES_3X3.get(axes).expect("map contains all combinations")
    }
}

// generates a two staged mapping
// symmetry axis -> index -> canonical index
fn generate_axis_to_canonical_indices<Ctx: GridSymmetryAxisContext>() -> HashMap<GridSymmetryAxes, Vec<usize>> {
    GridSymmetryAxis::iter().powerset().map(|axis| {
        let axis_set = axis.into_iter().collect::<EnumSet<_>>();
        // for each board index compute the canonical index (considering the symmetries)
        let canonical_indices = (0..=8).into_iter().map(|original_index| {
            // fixpoint iteration to find all indices that are equivalent to the given one
            let mut current = HashSet::new();
            current.insert(original_index);
            let mut old = current.clone();

            loop {
                for index in &old {
                    current.extend(axis_set.iter().flat_map(|axis| Ctx::expand_index(axis, *index)))
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
    }).collect()
}

lazy_static! {
    static ref AXIS_TO_CANONICAL_INDICES_3X3: HashMap<GridSymmetryAxes, Vec<usize>> = generate_axis_to_canonical_indices::<GridSymmetryAxisContext3x3>();
}

#[derive(Debug)]
pub struct GridSymmetry<Ctx: GridSymmetryAxisContext> {
    axes: GridSymmetryAxes,
    canonical_index: &'static Vec<usize>,
    ctx: PhantomData<Ctx>,
}

impl<Ctx: GridSymmetryAxisContext> Clone for GridSymmetry<Ctx> {
    fn clone(&self) -> Self {
        Self { axes: self.axes, canonical_index: self.canonical_index, ctx: self.ctx }
    }
}

impl<Ctx: GridSymmetryAxisContext> PartialEq for GridSymmetry<Ctx> {
    fn eq(&self, other: &Self) -> bool {
        self.axes == other.axes
    }
}

impl<Ctx: GridSymmetryAxisContext> Eq for GridSymmetry<Ctx> {}

impl<Ctx: GridSymmetryAxisContext> GridSymmetry<Ctx> {
    pub fn new<A>(axes: A) -> Self where A: Into<GridSymmetryAxes> {
        let set = axes.into();
        let canonical_index = Ctx::canonicalization_mapping(&set);
        Self { axes: set, canonical_index, ctx: PhantomData::default() }
    }

    pub fn none() -> Self {
        Self::new(EnumSet::empty())
    }
}

impl<Ctx: GridSymmetryAxisContext> Symmetry<usize> for GridSymmetry<Ctx> {
    fn canonicalize(&self, target: &usize) -> usize {
        debug_assert!(*target <= 8);
        self.canonical_index[*target]
    }

    fn expand(&self, normalised: &usize) -> Vec<usize> {
        self.axes.iter().flat_map(|axis| Ctx::expand_index(axis, *normalised)).collect()
    }
}

pub type GridSymmetry3x3 = GridSymmetry<GridSymmetryAxisContext3x3>;
pub type SymmetricMove3x3 = SymmetricMove<usize, GridSymmetry3x3>;

impl<C: Eq> From<&[C; 9]> for GridSymmetry3x3 {
    fn from(cells: &[C; 9]) -> Self {
        let axes = GridSymmetryAxis::iter().filter(|symmetry| {
            GridSymmetryAxisContext3x3::symmetric_indices(*symmetry).iter().all(|(f, s)| cells[*f] == cells[*s])
        }).collect::<GridSymmetryAxes>();
        GridSymmetry3x3::new(axes)
    }
}

impl GridSymmetry3x3 {
    pub fn is_same<C: Eq + Clone>(cells1: &[C; 9], cells2: &[C; 9]) -> bool {
        if cells1 == cells2 {
            return true;
        }
        // TODO: also find cases where a combination of symmetries is required
        GridSymmetryAxis::iter().any(|axis| {
            GridSymmetryAxisContext3x3::symmetric_indices(axis).iter().all(|(f, s)| cells1[*f] == cells2[*s])
        })
    }
}

#[cfg(test)]
mod test {
    use enumset::EnumSet;
    use itertools::Itertools;
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

    #[test]
    fn grid_symmetry3x3is_same() {
        let cells1 = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let cells2 = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert!(GridSymmetry3x3::is_same(&cells1, &cells2));
        
        let all_corner_boards = [
            [
                1, 0, 0,
                0, 0, 0,
                0, 0, 0
            ],
            [
                0, 0, 1,
                0, 0, 0,
                0, 0, 0
            ],
            [
                0, 0, 0,
                0, 0, 0,
                0, 0, 1
            ],
            [

                0, 0, 0,
                0, 0, 0,
                1, 0, 0
            ]
        ];
        all_corner_boards.iter().combinations(2).for_each(|pair| {
            assert!(GridSymmetry3x3::is_same(&pair[0], &pair[1]));
        });
        
        // let cells1 = [
        //     1, 1, 0,
        //     0, 0, 0,
        //     0, 0, 0
        // ];
        // let cells2 = [
        //     0, 0, 0,
        //     0, 0, 0,
        //     0, 1, 1
        // ];
        // assert!(GridSymmetry3x3::is_same(&cells1, &cells2));
    }
}