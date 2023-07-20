use std::ops::Deref;

use alot::{LotId, OrderedLots};
use gooey_core::math::units::UPx;
use gooey_core::math::Size;
use gooey_raster::ConstraintLimit;

use crate::flex::{FlexDimension, Orientation};

pub struct Flex {
    children: OrderedLots<FlexDimension>,
    layouts: Vec<FlexLayout>,
    pub other: UPx,
    total_weights: u32,
    allocated_space: UPx,
    fractional: Vec<(LotId, u8)>,
    measured: Vec<LotId>,
    pub orientation: Orientation,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FlexLayout {
    pub offset: UPx,
    pub size: UPx,
}

impl Flex {
    pub const fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            children: OrderedLots::new(),
            layouts: Vec::new(),
            other: UPx(0),
            total_weights: 0,
            allocated_space: UPx(0),
            fractional: Vec::new(),
            measured: Vec::new(),
        }
    }

    pub fn push(&mut self, child: FlexDimension) {
        self.insert(self.len(), child);
    }

    // pub fn remove(&mut self, index: usize) -> FlexDimension {
    //     let (id, dimension) = self.children.remove_by_index(index).expect("invalid index");
    //     self.layouts.remove(index);

    //     match dimension {
    //         FlexDimension::FitContent => {
    //             self.measured.retain(|&measured| measured != id);
    //         }
    //         FlexDimension::Fractional { weight } => {
    //             self.fractional.retain(|(measured, _)| *measured != id);
    //             self.total_weights -= u32::from(weight);
    //         }
    //         FlexDimension::Exact(size) => {
    //             self.allocated_space -= size;
    //         }
    //     }

    //     dimension
    // }

    pub fn insert(&mut self, index: usize, child: FlexDimension) {
        let id = self.children.insert(index, child);
        let layout = match child {
            FlexDimension::FitContent => {
                self.measured.push(id);
                UPx(0)
            }
            FlexDimension::Fractional { weight } => {
                self.total_weights += u32::from(weight);
                self.fractional.push((id, weight));
                UPx(0)
            }
            FlexDimension::Exact(size) => {
                self.allocated_space += size;
                size
            }
        };
        self.layouts.insert(
            index,
            FlexLayout {
                offset: UPx(0),
                size: layout,
            },
        );
    }

    pub fn update(
        &mut self,
        available: Size<ConstraintLimit>,
        mut measure: impl FnMut(usize, Size<ConstraintLimit>) -> Size<UPx>,
    ) -> Size<UPx> {
        let (space_constraint, other_constraint) = self.orientation.split_size(available);
        let available_space = space_constraint.max();
        let mut remaining = available_space.saturating_sub(self.allocated_space);

        // Measure the children that fit their content
        for &id in &self.measured {
            let index = self.children.index_of_id(id).expect("child not found");
            let (measured, _) = self.orientation.split_size(measure(
                index,
                self.orientation
                    .make_size(ConstraintLimit::ClippedAfter(remaining), other_constraint),
            ));
            self.layouts[index].size = measured;
            remaining = remaining.saturating_sub(measured);
        }

        // Measure the weighted children within the remaining space
        if self.total_weights > 0 {
            let space_per_weight = remaining / self.total_weights;
            remaining %= self.total_weights;
            for (fractional_index, &(id, weight)) in self.fractional.iter().enumerate() {
                let index = self.children.index_of_id(id).expect("child not found");
                let size = space_per_weight * u32::from(weight);
                self.layouts[index].size = size;

                // If we have fractional amounts remaining, divide the pixels
                if remaining > 0 {
                    let from_end = u32::try_from(self.fractional.len() - fractional_index)
                        .expect("too many items");
                    if remaining >= from_end {
                        let amount = (remaining + from_end - 1) / from_end;
                        remaining -= amount;
                        self.layouts[index].size += amount;
                    }
                }
            }
        }

        // Now that we know the constrained sizes, we can measure the children
        // to get the other measurement using the constrainted measurement.
        self.other = UPx(0);
        let mut offset = UPx(0);
        for index in 0..self.children.len() {
            self.layouts[index].offset = offset;
            offset += self.layouts[index].size;
            let (_, measured) = self.orientation.split_size(measure(
                index,
                self.orientation.make_size(
                    ConstraintLimit::Known(self.layouts[index].size),
                    other_constraint,
                ),
            ));
            self.other = self.other.max(measured);
        }

        self.other = match other_constraint {
            ConstraintLimit::Known(max) => self.other.max(max),
            ConstraintLimit::ClippedAfter(clip_limit) => self.other.min(clip_limit),
        };

        self.orientation.make_size(available_space, self.other)
    }
}

impl Deref for Flex {
    type Target = [FlexLayout];

    fn deref(&self) -> &Self::Target {
        &self.layouts
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use gooey_core::math::units::UPx;
    use gooey_core::math::Size;
    use gooey_raster::ConstraintLimit;

    use super::{Flex, FlexDimension, Orientation};

    struct Child {
        size: UPx,
        dimension: FlexDimension,
        other: UPx,
        divisible_by: Option<UPx>,
    }

    impl Child {
        pub fn new(size: impl Into<UPx>, other: impl Into<UPx>) -> Self {
            Self {
                size: size.into(),
                dimension: FlexDimension::FitContent,
                other: other.into(),
                divisible_by: None,
            }
        }

        pub fn fixed_size(mut self, size: UPx) -> Self {
            self.dimension = FlexDimension::Exact(size);
            self
        }

        pub fn weighted(mut self, weight: u8) -> Self {
            self.dimension = FlexDimension::Fractional { weight };
            self
        }

        pub fn divisible_by(mut self, split_at: impl Into<UPx>) -> Self {
            self.divisible_by = Some(split_at.into());
            self
        }
    }

    fn assert_measured_children_in_orientation(
        orientation: Orientation,
        children: &[Child],
        available: Size<ConstraintLimit>,
        expected: &[UPx],
        expected_size: Size<UPx>,
    ) {
        assert_eq!(children.len(), expected.len());
        let mut flex = Flex::new(orientation);
        for child in children {
            flex.push(child.dimension);
        }

        let computed_size = flex.update(available, |index, constraints| {
            let (measured_constraint, _other_constraint) = orientation.split_size(constraints);
            let child = &children[index];
            let maximum_measured = measured_constraint.max();
            let (measured, other) = match (child.size.cmp(&maximum_measured), child.divisible_by) {
                (Ordering::Greater, Some(divisible_by)) => {
                    let available_divided = maximum_measured / divisible_by;
                    let rows = ((child.size + divisible_by - 1) / divisible_by + available_divided
                        - 1)
                        / available_divided;
                    (available_divided * divisible_by, child.other * rows)
                }
                _ => (child.size, child.other),
            };
            orientation.make_size(measured, other)
        });
        assert_eq!(computed_size, expected_size);
        let mut offset = UPx(0);
        for ((index, &child), &expected) in flex.iter().enumerate().zip(expected) {
            assert_eq!(
                child.size,
                expected,
                "child {index} measured to {}, expected {}",
                child.size,
                expected // TODO Display for UPx
            );
            assert_eq!(child.offset, offset);
            offset += child.size;
        }
    }

    fn assert_measured_children(
        children: &[Child],
        main_constraint: ConstraintLimit,
        other_constraint: ConstraintLimit,
        expected: &[UPx],
        expected_measured: UPx,
        expected_other: UPx,
    ) {
        assert_measured_children_in_orientation(
            Orientation::Row,
            children,
            Orientation::Row.make_size(main_constraint, other_constraint),
            expected,
            Orientation::Row.make_size(expected_measured, expected_other),
        );
        assert_measured_children_in_orientation(
            Orientation::Column,
            children,
            Orientation::Column.make_size(main_constraint, other_constraint),
            expected,
            Orientation::Column.make_size(expected_measured, expected_other),
        );
    }

    #[test]
    fn size_to_fit() {
        assert_measured_children(
            &[Child::new(3, 1), Child::new(3, 1), Child::new(3, 1)],
            ConstraintLimit::ClippedAfter(UPx(10)),
            ConstraintLimit::ClippedAfter(UPx(10)),
            &[UPx(3), UPx(3), UPx(3)],
            UPx(10),
            UPx(1),
        );
    }

    #[test]
    fn wrapping() {
        // This tests some fun rounding edge cases. Because the total weights is
        // 4 and the size is 10, we have inexact math to determine the pixel
        // width of each child.
        //
        // In this particular example, it shows the weights are clamped so that
        // each is credited for 2px. This is why the first child ends up with
        // 4px. However, with 4 total weight, that leaves a remaining 2px to be
        // assigned. The flex algorithm divides the remaining pixels amongst the
        // remaining children.
        assert_measured_children(
            &[
                Child::new(20, 1).divisible_by(3).weighted(2),
                Child::new(3, 1).weighted(1),
                Child::new(3, 1).weighted(1),
            ],
            ConstraintLimit::Known(UPx(10)),
            ConstraintLimit::ClippedAfter(UPx(10)),
            &[UPx(4), UPx(3), UPx(3)],
            UPx(10),
            UPx(7), // 20 / 3 = 6.666, rounded up is 7
        );
        // Same as above, but with an 11px box. This creates a leftover of 3 px
        // (11 % 4), adding 1px to all three children.
        assert_measured_children(
            &[
                Child::new(20, 1).divisible_by(3).weighted(2),
                Child::new(3, 1).weighted(1),
                Child::new(3, 1).weighted(1),
            ],
            ConstraintLimit::Known(UPx(11)),
            ConstraintLimit::ClippedAfter(UPx(11)),
            &[UPx(5), UPx(3), UPx(3)],
            UPx(11),
            UPx(7), // 20 / 3 = 6.666, rounded up is 7
        );
        // 12px box. This creates no leftover.
        assert_measured_children(
            &[
                Child::new(20, 1).divisible_by(3).weighted(2),
                Child::new(3, 1).weighted(1),
                Child::new(3, 1).weighted(1),
            ],
            ConstraintLimit::Known(UPx(12)),
            ConstraintLimit::ClippedAfter(UPx(12)),
            &[UPx(6), UPx(3), UPx(3)],
            UPx(12),
            UPx(4), // 20 / 6 = 3.666, rounded up is 4
        );
        // 13px box. This creates a leftover of 1 px (13 % 4), adding 1px only
        // to the final child
        assert_measured_children(
            &[
                Child::new(20, 1).divisible_by(3).weighted(2),
                Child::new(3, 1).weighted(1),
                Child::new(3, 1).weighted(1),
            ],
            ConstraintLimit::Known(UPx(13)),
            ConstraintLimit::ClippedAfter(UPx(13)),
            &[UPx(6), UPx(3), UPx(4)],
            UPx(13),
            UPx(4), // 20 / 6 = 3.666, rounded up is 4
        );
    }

    #[test]
    fn fixed_size() {
        assert_measured_children(
            &[
                Child::new(3, 1).fixed_size(UPx(7)),
                Child::new(3, 1).weighted(1),
                Child::new(3, 1).weighted(1),
            ],
            ConstraintLimit::Known(UPx(15)),
            ConstraintLimit::ClippedAfter(UPx(15)),
            &[UPx(7), UPx(4), UPx(4)],
            UPx(15),
            UPx(1),
        );
    }
}
