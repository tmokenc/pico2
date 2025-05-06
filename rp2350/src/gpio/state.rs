/**
 * @file gpio/state.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief Definition of the state of a GPIO pin
 */
use super::FunctionSelect;

#[derive(Debug, Clone, Copy)]
pub enum OutputState {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum InputState {
    PullUp,
    PullDown,
    #[default]
    Floating,
    BusKeeper,
}

#[derive(Debug, Clone, Copy)]
pub enum PinState {
    Output(OutputState, FunctionSelect),
    Input(InputState),
}

impl PinState {
    pub fn is_high(&self) -> bool {
        matches!(self, Self::Output(OutputState::High, _))
    }
}
