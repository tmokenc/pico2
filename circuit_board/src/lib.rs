pub trait Component {
    /// Returns the number of pins for this component.
    fn num_pins(&self) -> usize;

    /// Returns a reference to the pin at the given index.
    fn get_pin(&self, pin_idx: usize) -> &Pin;

    /// Returns a mutable reference to the pin at the given index.
    fn get_pin_mut(&mut self, pin_idx: usize) -> &mut Pin;

    /// Specifies whether a given pin is an input pin.
    fn is_input_pin(&self, pin_idx: usize) -> bool;

    /// Notify the component that one of its input pins has changed.
    fn notify_pin_change(&mut self, pin_id: usize, new_voltage: f32, new_amp: f32);
}
