//! Provides options to configure the behavior of reconnect-stream items,
//! specifically related to reconnect behavior.

use std::sync::Arc;
use std::time::Duration;

pub type DurationIterator = Box<dyn Iterator<Item = Duration> + Send + Sync>;

/// User specified options that control the behavior of the [ReconnectStream](crate::ReconnectStream) upon disconnect.
#[derive(Clone)]
pub struct ReconnectOptions {
    /// Represents a function that generates an Iterator
    /// to schedule the wait between reconnection attempts.
    pub retries_to_attempt_fn: Arc<dyn Fn() -> DurationIterator + Send + Sync>,

    /// If this is set to true, if the initial connect method of the [ReconnectStream](crate::ReconnectStream) item fails,
    /// then no further reconnects will be attempted
    pub exit_if_first_connect_fails: bool,

    /// Invoked when the [ReconnectStream](crate::ReconnectStream) establishes a connection
    pub on_connect_callback: Arc<dyn Fn() + Send + Sync>,

    /// Invoked when the [ReconnectStream](crate::ReconnectStream) loses its active connection
    pub on_disconnect_callback: Arc<dyn Fn() + Send + Sync>,

    /// Invoked when the [ReconnectStream](crate::ReconnectStream) fails a connection attempt
    pub on_connect_fail_callback: Arc<dyn Fn() + Send + Sync>,
}

impl ReconnectOptions {
    /// By default, the [ReconnectStream](crate::ReconnectStream) will not try to reconnect if the first connect attempt fails.
    /// By default, the retries iterator waits longer and longer between reconnection attempts,
    /// until it eventually perpetually tries to reconnect every 30 minutes.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ReconnectOptions {
            retries_to_attempt_fn: Arc::new(get_standard_reconnect_strategy),
            exit_if_first_connect_fails: true,
            on_connect_callback: Arc::new(|| {}),
            on_disconnect_callback: Arc::new(|| {}),
            on_connect_fail_callback: Arc::new(|| {}),
        }
    }

    /// This convenience function allows the user to provide any function that returns a value
    /// that is convertible into an iterator, such as an actual iterator or a Vec.
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stream_reconnect::ReconnectOptions;
    ///
    /// // With the below vector, the ReconnectStream item will try to reconnect three times,
    /// // waiting 2 seconds between each attempt. Once all three tries are exhausted,
    /// // it will stop attempting.
    /// let options = ReconnectOptions::new().with_retries_generator(|| {
    ///     vec![
    ///         Duration::from_secs(2),
    ///         Duration::from_secs(2),
    ///         Duration::from_secs(2),
    ///     ]
    /// });
    /// ```
    pub fn with_retries_generator<F, I, IN>(mut self, retries_generator: F) -> Self
    where
        F: 'static + Send + Sync + Fn() -> IN,
        I: 'static + Send + Sync + Iterator<Item = Duration>,
        IN: IntoIterator<IntoIter = I, Item = Duration>,
    {
        self.retries_to_attempt_fn = Arc::new(move || Box::new(retries_generator().into_iter()));
        self
    }

    pub fn with_exit_if_first_connect_fails(mut self, value: bool) -> Self {
        self.exit_if_first_connect_fails = value;
        self
    }

    pub fn with_on_connect_callback(mut self, cb: impl Fn() + 'static + Send + Sync) -> Self {
        self.on_connect_callback = Arc::new(cb);
        self
    }

    pub fn with_on_disconnect_callback(mut self, cb: impl Fn() + 'static + Send + Sync) -> Self {
        self.on_disconnect_callback = Arc::new(cb);
        self
    }

    pub fn with_on_connect_fail_callback(mut self, cb: impl Fn() + 'static + Send + Sync) -> Self {
        self.on_connect_fail_callback = Arc::new(cb);
        self
    }
}

fn get_standard_reconnect_strategy() -> DurationIterator {
    let initial_attempts = vec![
        Duration::from_secs(5),
        Duration::from_secs(10),
        Duration::from_secs(20),
        Duration::from_secs(30),
        Duration::from_secs(40),
        Duration::from_secs(50),
        Duration::from_secs(60),
        Duration::from_secs(60 * 2),
        Duration::from_secs(60 * 5),
        Duration::from_secs(60 * 10),
        Duration::from_secs(60 * 20),
    ];

    let repeat = std::iter::repeat(Duration::from_secs(60 * 30));

    let forever_iterator = initial_attempts.into_iter().chain(repeat);

    Box::new(forever_iterator)
}
