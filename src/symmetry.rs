use strum_macros::EnumIter;

pub trait Symmetries<T> {
    fn normalise(&self, target: &T) -> T;
    fn expand(&self, normalised: &T) -> Vec<T>;
}

#[derive(EnumIter, Eq, PartialEq, Debug, Copy, Clone)]
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
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct GridSymmetry3x3 {
    axes: Vec<GridSymmetryAxis>,
}

impl Symmetries<usize> for GridSymmetry3x3 {
    fn normalise(&self, target: &usize) -> usize {
        fn normalise_by_axes(index: usize, axis: GridSymmetryAxis) -> usize {
            return *axis.symmetric_indices_3x3().iter()
                .find(|(_, second)| index == *second)
                .map(|(first, _)| first)
                .unwrap_or(&index);
        }
        return self.axes.iter().fold(*target, |index, axis| normalise_by_axes(index, *axis));
    }

    fn expand(&self, normalised: &usize) -> Vec<usize> {
        todo!()
    }
}