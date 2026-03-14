use std::{collections::VecDeque, sync::Mutex};

pub struct SafeQueue<T> {
    queue: Mutex<VecDeque<T>>,
}

impl<T> SafeQueue<T> {
    pub fn new() -> Self {
        SafeQueue {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn enqueue(&self, value: T) -> Result<(), String> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|_| "Failed to lock queue mutex")?;

        queue.push_back(value);

        Ok(())
    }

    pub fn pop(&self) -> Result<Option<T>, String> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|_| "Failed to lock queue mutex")?;

        Ok(queue.pop_front())
    }

    pub fn len(&self) -> Result<usize, String> {
        let queue = self.queue.lock().map_err(|_| "Failed to get lock")?;

        Ok(queue.len())
    }

    pub fn is_empty(&self) -> Result<bool, String> {
        let queue = self.queue.lock().map_err(|_| "Failed to get lock")?;
        Ok(queue.is_empty())
    }
}

