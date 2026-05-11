/// A simple double-buffered event channel.
///
/// Events sent during the current frame are available via [`read`](Events::read),
/// which yields both the current frame's events and the previous frame's events.
/// Calling [`update`](Events::update) at the start of each frame advances the
/// double buffer, moving current events to the previous slot and clearing the
/// current slot.
///
/// This ensures that events are readable for at least one full frame after
/// being sent, even if the reader runs before the sender within the same frame.
pub struct Events<T> {
    /// Events sent during the current frame/tick.
    events: Vec<T>,
    /// Events from the previous frame/tick (the "back buffer").
    prev_events: Vec<T>,
}

impl<T> Events<T> {
    /// Creates a new, empty event channel.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            prev_events: Vec::new(),
        }
    }

    /// Sends an event, making it available to readers this frame and next.
    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    /// Returns an iterator over all readable events (previous + current frame).
    ///
    /// Previous-frame events are yielded first, then current-frame events.
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.prev_events.iter().chain(self.events.iter())
    }

    /// Advances the double buffer: moves current events to the previous
    /// slot and clears the current slot.
    ///
    /// Should be called once per frame, typically at the very beginning
    /// of the frame (before any systems send or read events).
    pub fn update(&mut self) {
        // Swap current into previous, then clear current.
        std::mem::swap(&mut self.prev_events, &mut self.events);
        self.events.clear();
    }

    /// Returns the total number of readable events (current + previous).
    pub fn len(&self) -> usize {
        self.events.len() + self.prev_events.len()
    }

    /// Returns `true` if there are no readable events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.prev_events.is_empty()
    }

    /// Clears all events from both buffers.
    pub fn clear(&mut self) {
        self.events.clear();
        self.prev_events.clear();
    }
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience type for sending events. Wraps a mutable reference to [`Events<T>`].
pub struct EventWriter<'a, T> {
    events: &'a mut Events<T>,
}

impl<'a, T> EventWriter<'a, T> {
    /// Creates a new event writer.
    pub fn new(events: &'a mut Events<T>) -> Self {
        Self { events }
    }

    /// Sends an event.
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }
}

/// Convenience type for reading events. Wraps an immutable reference to [`Events<T>`].
pub struct EventReader<'a, T> {
    events: &'a Events<T>,
}

impl<'a, T> EventReader<'a, T> {
    /// Creates a new event reader.
    pub fn new(events: &'a Events<T>) -> Self {
        Self { events }
    }

    /// Returns an iterator over all readable events.
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.events.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_and_read() {
        let mut events = Events::<i32>::new();
        events.send(1);
        events.send(2);
        events.send(3);

        let collected: Vec<&i32> = events.read().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    #[test]
    fn read_empty_yields_nothing() {
        let events = Events::<String>::new();
        assert_eq!(events.read().count(), 0);
        assert!(events.is_empty());
    }

    #[test]
    fn update_moves_to_prev() {
        let mut events = Events::<i32>::new();
        events.send(10);
        events.send(20);

        events.update();

        // After update, previous events should still be readable.
        let collected: Vec<&i32> = events.read().collect();
        assert_eq!(collected, vec![&10, &20]);

        // Send new events after update.
        events.send(30);
        let collected: Vec<&i32> = events.read().collect();
        // Should see both prev (10, 20) and current (30).
        assert_eq!(collected, vec![&10, &20, &30]);
    }

    #[test]
    fn double_update_clears_old() {
        let mut events = Events::<i32>::new();
        events.send(1);

        events.update(); // 1 moves to prev.
        events.send(2);

        events.update(); // 2 moves to prev, 1 is gone.

        let collected: Vec<&i32> = events.read().collect();
        assert_eq!(collected, vec![&2]);
    }

    #[test]
    fn triple_update_clears_all() {
        let mut events = Events::<i32>::new();
        events.send(1);
        events.update();
        events.update();
        events.update();

        assert!(events.is_empty());
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn len_tracks_both_buffers() {
        let mut events = Events::<i32>::new();
        events.send(1);
        events.send(2);
        assert_eq!(events.len(), 2);

        events.update();
        events.send(3);
        // 2 in prev + 1 in current = 3
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn clear_empties_everything() {
        let mut events = Events::<i32>::new();
        events.send(1);
        events.update();
        events.send(2);

        events.clear();
        assert!(events.is_empty());
        assert_eq!(events.read().count(), 0);
    }

    #[test]
    fn event_writer_sends() {
        let mut events = Events::<String>::new();
        {
            let mut writer = EventWriter::new(&mut events);
            writer.send(String::from("hello"));
            writer.send(String::from("world"));
        }
        let collected: Vec<&String> = events.read().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn event_reader_reads() {
        let mut events = Events::<i32>::new();
        events.send(42);
        let reader = EventReader::new(&events);
        let collected: Vec<&i32> = reader.read().collect();
        assert_eq!(collected, vec![&42]);
    }

    #[derive(Debug, PartialEq)]
    struct CollisionEvent {
        entity_a: u32,
        entity_b: u32,
    }

    #[test]
    fn custom_event_type() {
        let mut events = Events::<CollisionEvent>::new();
        events.send(CollisionEvent {
            entity_a: 1,
            entity_b: 2,
        });
        events.send(CollisionEvent {
            entity_a: 3,
            entity_b: 4,
        });

        let collected: Vec<&CollisionEvent> = events.read().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(
            collected[0],
            &CollisionEvent {
                entity_a: 1,
                entity_b: 2
            }
        );
    }
}
