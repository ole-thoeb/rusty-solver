use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
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

#[derive(Debug)]
pub struct GridSymmetryAxisContext9x9;

impl GridSymmetryAxisContext for GridSymmetryAxisContext9x9 {
    type SymmetricPairs = [(usize, usize); 36];

    fn symmetric_indices(axis: GridSymmetryAxis) -> Self::SymmetricPairs {
        match axis {
            GridSymmetryAxis::Vertical => [(0, 72), (1, 73), (2, 74), (3, 75), (4, 76), (5, 77), (6, 78), (7, 79), (8, 80), (9, 63), (10, 64), (11, 65), (12, 66), (13, 67), (14, 68), (15, 69), (16, 70), (17, 71), (18, 54), (19, 55), (20, 56), (21, 57), (22, 58), (23, 59), (24, 60), (25, 61), (26, 62), (27, 45), (28, 46), (29, 47), (30, 48), (31, 49), (32, 50), (33, 51), (34, 52), (35, 53)],
            GridSymmetryAxis::Horizontal => [(0, 8), (9, 17), (18, 26), (27, 35), (36, 44), (45, 53), (54, 62), (63, 71), (72, 80), (1, 7), (10, 16), (19, 25), (28, 34), (37, 43), (46, 52), (55, 61), (64, 70), (73, 79), (2, 6), (11, 15), (20, 24), (29, 33), (38, 42), (47, 51), (56, 60), (65, 69), (74, 78), (3, 5), (12, 14), (21, 23), (30, 32), (39, 41), (48, 50), (57, 59), (66, 68), (75, 77)],
            GridSymmetryAxis::DiagonalTopBottom => [(1, 9), (2, 18), (3, 27), (4, 36), (5, 45), (6, 54), (7, 63), (8, 72), (11, 19), (12, 28), (13, 37), (14, 46), (15, 55), (16, 64), (17, 73), (21, 29), (22, 38), (23, 47), (24, 56), (25, 65), (26, 74), (31, 39), (32, 48), (33, 57), (34, 66), (35, 75), (41, 49), (42, 58), (43, 67), (44, 76), (51, 59), (52, 68), (53, 77), (61, 69), (62, 78), (71, 79)],
            GridSymmetryAxis::DiagonalBottomTop => [(0, 80), (1, 71), (2, 62), (3, 53), (4, 44), (5, 35), (6, 26), (7, 17), (9, 79), (10, 70), (11, 61), (12, 52), (13, 43), (14, 34), (15, 25), (18, 78), (19, 69), (20, 60), (21, 51), (22, 42), (23, 33), (27, 77), (28, 68), (29, 59), (30, 50), (31, 41), (36, 76), (37, 67), (38, 58), (39, 49), (45, 75), (46, 66), (47, 57), (54, 74), (55, 65), (63, 73)],
        }
    }

    fn canonicalization_mapping(axes: &GridSymmetryAxes) -> &'static Vec<usize> {
        AXIS_TO_CANONICAL_INDICES_9X9.get(axes).expect("map contains all combinations")
    }
}

#[test]
fn generate9x9() {
    // vertical
    // let initial = vec![(0, 72), (9, 63), (18, 54), (27, 45)];
    // let all = initial.iter().flat_map(|(f, s)| (0..=8).into_iter().map(move |i| (f + i, s + i))).collect::<Vec<_>>();
    // horizontal
    // let initial = vec![(0, 8), (1, 7), (2, 6), (3, 5)];
    // let all = initial.iter().flat_map(|(f, s)| (0..=8).into_iter().map(move |i| (f + i * 9, s + i * 9))).collect::<Vec<_>>();
    // DiagonalTopBottom
    // let initial = vec![1, 11, 21, 31, 41, 51, 61, 71];
    // let all = initial.iter().enumerate().flat_map(|(i, start)| (0..=(7 - i)).into_iter().map(move |i| {
    //     let f = start + i;
    //     (f, f + (i + 1) * 8)
    // })).collect::<Vec<_>>();
    let initial = vec![0, 9, 18, 27, 36, 45, 54, 63];
    let all = initial.iter().enumerate().flat_map(|(i, start)| {
        let (f, s) = (start, 80 - i);
        (0..=(7 - i)).into_iter().map(move |i| {
            (f + i, s - i * 9)
        })
    }).collect::<Vec<_>>();
    println!("{:?}", all)
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
    static ref AXIS_TO_CANONICAL_INDICES_9X9: HashMap<GridSymmetryAxes, Vec<usize>> = generate_axis_to_canonical_indices::<GridSymmetryAxisContext9x9>();
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

pub type GridSymmetry9x9 = GridSymmetry<GridSymmetryAxisContext9x9>;
pub type SymmetricMove9x9 = SymmetricMove<usize, GridSymmetry9x9>;

impl<C: Eq> From<&[C; 81]> for GridSymmetry9x9 {
    fn from(cells: &[C; 81]) -> Self {
        let axes = GridSymmetryAxis::iter().filter(|symmetry| {
            GridSymmetryAxisContext9x9::symmetric_indices(*symmetry).iter().all(|(f, s)| cells[*f] == cells[*s])
        }).collect::<GridSymmetryAxes>();
        GridSymmetry9x9::new(axes)
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