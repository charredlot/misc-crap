use super::event::{Event, EventOrder, EventQueue, EventTime, Priority};
use super::ActionPoints;

struct Turn {
    time: EventTime,
    ap: ActionPoints,
}

fn insert_turn(q: &mut EventQueue, turn: Turn) {
    q.insert(
        turn.time,
        Priority::Turn,
        Box::new(turn),
    );
}

impl Event for Turn {
    fn needs_input(&self) -> bool {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_event_queue_panic() {
        let mut q = EventQueue::new();
        for _ in 0..2 {
            insert_turn(&mut q, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
            });
        }

        q.advance();

        /* this should fail since it should be waiting for input */
        q.advance();
    }

    #[test]
    fn test_event_queue() {
        let mut q = EventQueue::new();
        for _ in 0..2 {
            insert_turn(&mut q, Turn{
                time: 10 as EventTime,
                ap: 0 as ActionPoints,
            });
        }

        let (_, e) = q.advance();
        assert!(e.needs_input());
        q.input_processed();
        q.advance();
    }
}
