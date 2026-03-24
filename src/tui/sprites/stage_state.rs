/// Simple stage state for the chicken party view.
/// Agent data comes from the StoreSnapshot each frame --
/// no persistent sprite positions or maze needed.
pub struct StageState {
    pub initialized: bool,
    pub tick: usize,
}

impl StageState {
    pub fn new() -> Self {
        StageState {
            initialized: false,
            tick: 0,
        }
    }
}
