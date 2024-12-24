use std::collections::HashMap;

pub struct Stats {
    pub executed_instructions: HashMap<String, u32>,
    pub executed_cycles: u32,
    pub branch_predicts: u32,
    pub miss_predicts: u32,
}
