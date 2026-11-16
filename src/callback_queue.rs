//! Queue that FFI callbacks hand events to.
//!
//! DNS-SD callbacks run on threads we don't own: `DNSServiceProcessResult`'s caller on Apple/Bonjour, and the DNS
//! API's own threads on Windows. Blocking there can hang the caller or a later cancel, and dropping an event loses a
//! discovery for good. So the queue is unbounded and pushing never blocks; events are small and bounded by the
//! services on the network, and the receiver drains them.

use std::sync::mpsc::{channel, Receiver, SendError, Sender};

/// Sending half handed to an FFI callback as its context, see [`callback_queue`]
#[derive(Debug)]
pub(crate) struct CallbackSender<T> {
    tx: Sender<T>,
}

impl<T> CallbackSender<T> {
    /// Queues an event without blocking. Only fails once the receiver is gone (the browser was dropped), so the
    /// event has nobody left to go to
    pub(crate) fn push(&self, event: T) -> Result<(), SendError<T>> {
        self.tx.send(event)
    }
}

/// Creates a queue for a callback to push into without ever blocking or dropping events
pub(crate) fn callback_queue<T>() -> (CallbackSender<T>, Receiver<T>) {
    let (tx, rx) = channel();
    (CallbackSender { tx }, rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn push_never_blocks_with_slow_consumer() {
        let (tx, rx) = callback_queue::<usize>();
        let consumer = std::thread::spawn(move || {
            let mut got = Vec::new();
            while let Ok(i) = rx.recv() {
                std::thread::sleep(Duration::from_micros(200));
                got.push(i);
            }
            got
        });
        let start = Instant::now();
        for i in 0..1_000 {
            tx.push(i).expect("receiver alive");
        }
        // a slow consumer takes >= 200ms to drain, pushing must not wait for it
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "push blocked: {:?}",
            start.elapsed()
        );
        drop(tx);
        assert_eq!(consumer.join().unwrap(), (0..1_000).collect::<Vec<_>>());
    }

    #[test]
    fn push_fails_once_receiver_dropped() {
        let (tx, rx) = callback_queue::<u8>();
        drop(rx);
        assert!(tx.push(1).is_err());
    }
}
