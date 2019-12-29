
/* Represents an inputted command for a unit (cpu or player controlled) */
pub trait Input {
}

pub struct EndTurnInput {}

impl Input for EndTurnInput {}
