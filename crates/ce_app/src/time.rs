use std::time::{Duration, Instant};

/// Tracks frame timing. Inserted as a Resource in the World.
///
/// Call [`Time::update`] once per frame to compute the delta from the
/// previous frame and accumulate elapsed time.
pub struct Time {
    startup: Instant,
    last_frame: Instant,
    delta: Duration,
    delta_seconds: f32,
    elapsed: Duration,
    frame_count: u64,
}

impl Time {
    /// Creates a new `Time` with the current instant as both startup and
    /// last-frame timestamp. Delta starts at zero.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            startup: now,
            last_frame: now,
            delta: Duration::ZERO,
            delta_seconds: 0.0,
            elapsed: Duration::ZERO,
            frame_count: 0,
        }
    }

    /// Updates timing data. Must be called exactly once per frame, before
    /// systems run.
    ///
    /// Computes the delta since the last call to `update`, advances elapsed
    /// time, and increments the frame counter.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame);
        self.delta_seconds = self.delta.as_secs_f32();
        self.elapsed = now.duration_since(self.startup);
        self.last_frame = now;
        self.frame_count += 1;
    }

    /// Duration between the last two frames.
    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Delta as `f32` seconds — the most common form used in gameplay code.
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Total wall-clock time since the `Time` resource was created.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Elapsed time as `f32` seconds.
    #[inline]
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    /// Number of completed frames (incremented each time `update` is called).
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed-timestep configuration for deterministic simulation stages.
///
/// Systems in [`CoreStage::FixedUpdate`] conceptually run at this fixed rate.
/// The accumulator pattern (not yet wired) will consume `step` increments
/// from real elapsed time, running the fixed stage zero or more times per
/// frame.
pub struct FixedTime {
    /// The fixed timestep duration.
    step: Duration,
    /// Accumulated real time waiting to be consumed.
    accumulator: Duration,
}

impl FixedTime {
    /// Creates a `FixedTime` with the given step duration.
    pub fn new(step: Duration) -> Self {
        Self {
            step,
            accumulator: Duration::ZERO,
        }
    }

    /// Creates a `FixedTime` from a target tick rate in Hz.
    pub fn from_hz(hz: f64) -> Self {
        Self::new(Duration::from_secs_f64(1.0 / hz))
    }

    /// The fixed step duration.
    #[inline]
    pub fn step(&self) -> Duration {
        self.step
    }

    /// The fixed step as `f32` seconds.
    #[inline]
    pub fn step_seconds(&self) -> f32 {
        self.step.as_secs_f32()
    }

    /// Current accumulated real time.
    #[inline]
    pub fn accumulator(&self) -> Duration {
        self.accumulator
    }

    /// Adds real elapsed time to the accumulator.
    pub fn accumulate(&mut self, delta: Duration) {
        self.accumulator += delta;
    }

    /// Returns `true` and subtracts one step if the accumulator holds at
    /// least one full step. Returns `false` otherwise.
    pub fn expend(&mut self) -> bool {
        if self.accumulator >= self.step {
            self.accumulator -= self.step;
            true
        } else {
            false
        }
    }
}

impl Default for FixedTime {
    /// Defaults to 60 Hz.
    fn default() -> Self {
        Self::from_hz(60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_new_starts_at_zero() {
        let time = Time::new();
        assert_eq!(time.delta(), Duration::ZERO);
        assert_eq!(time.delta_seconds(), 0.0);
        assert_eq!(time.frame_count(), 0);
    }

    #[test]
    fn time_update_increments_frame_count() {
        let mut time = Time::new();
        time.update();
        assert_eq!(time.frame_count(), 1);
        time.update();
        assert_eq!(time.frame_count(), 2);
    }

    #[test]
    fn time_update_produces_nonnegative_delta() {
        let mut time = Time::new();
        // Small spin to ensure some measurable time passes.
        std::thread::sleep(Duration::from_millis(1));
        time.update();
        assert!(time.delta() > Duration::ZERO);
        assert!(time.delta_seconds() > 0.0);
    }

    #[test]
    fn time_elapsed_grows() {
        let mut time = Time::new();
        std::thread::sleep(Duration::from_millis(5));
        time.update();
        let e1 = time.elapsed();
        std::thread::sleep(Duration::from_millis(5));
        time.update();
        let e2 = time.elapsed();
        assert!(e2 > e1);
        assert!(time.elapsed_seconds() > 0.0);
    }

    #[test]
    fn fixed_time_default_is_60hz() {
        let ft = FixedTime::default();
        let expected = Duration::from_secs_f64(1.0 / 60.0);
        // Allow tiny floating-point mismatch.
        let diff = if ft.step() > expected {
            ft.step() - expected
        } else {
            expected - ft.step()
        };
        assert!(diff < Duration::from_nanos(100));
    }

    #[test]
    fn fixed_time_accumulate_and_expend() {
        let mut ft = FixedTime::from_hz(10.0); // 100ms step
        ft.accumulate(Duration::from_millis(250));
        assert!(ft.expend()); // 250 -> 150
        assert!(ft.expend()); // 150 -> 50
        assert!(!ft.expend()); // 50 < 100, cannot expend
    }

    #[test]
    fn fixed_time_step_seconds() {
        let ft = FixedTime::from_hz(30.0);
        let expected = 1.0f32 / 30.0;
        assert!((ft.step_seconds() - expected).abs() < 1e-6);
    }
}
