/**
 * @file /processor/hazard/branch_predictor.rs
 * @author Nguyen Le Duy
 * @date 31/03/2025
 * @brief A simple branch predictor that uses a last branch taken strategy.
 */

#[derive(Default)]
pub struct BranchPredictor {
    pub last_branch_taken: Option<u32>,
}

impl BranchPredictor {
    pub fn miss_predicted(&mut self, pc: u32, taken: bool) -> bool {
        if taken {
            if self.last_branch_taken == Some(pc) {
                // correctly predicted
                false
            } else {
                // mispredicted
                self.last_branch_taken = Some(pc);
                true
            }
        } else {
            if self.last_branch_taken == Some(pc) {
                // mispredicted
                self.last_branch_taken = None;
                true
            } else {
                // correctly predicted
                false
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_branch_predictor() {
        let mut predictor = BranchPredictor::default();

        assert_eq!(predictor.miss_predicted(0x1000, false), false);
        assert_eq!(predictor.miss_predicted(0x1000, true), true);
        assert_eq!(predictor.miss_predicted(0x1000, true), false);
        assert_eq!(predictor.miss_predicted(0x1000, false), true);
        assert_eq!(predictor.miss_predicted(0x1000, false), false);
        assert_eq!(predictor.miss_predicted(0x2000, true), true);
        assert_eq!(predictor.miss_predicted(0x2000, true), false);
        assert_eq!(predictor.miss_predicted(0x2000, false), true);
        assert_eq!(predictor.miss_predicted(0x2000, false), false);
    }

    #[test]
    fn test_branch_predictor_with_different_pcs() {
        let mut predictor = BranchPredictor::default();

        assert_eq!(predictor.miss_predicted(0x1000, false), false);
        assert_eq!(predictor.miss_predicted(0x2000, false), false);
        assert_eq!(predictor.miss_predicted(0x1000, true), true);
        assert_eq!(predictor.miss_predicted(0x2000, true), true);
        assert_eq!(predictor.miss_predicted(0x1000, false), false);
        assert_eq!(predictor.miss_predicted(0x2000, true), false);
        assert_eq!(predictor.miss_predicted(0x3000, true), true);
    }
}
