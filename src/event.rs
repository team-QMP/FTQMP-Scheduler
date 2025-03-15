use crate::job::JobID;
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EventType {
    RequestJob {
        job_id: JobID,
    },
    StartScheduling,
}

impl EventType {
    /// Return the priority between events occuring at the same time.
    /// The default priority value is 0.
    #[allow(unreachable_patterns)]
    pub fn priority(&self) -> i32 {
        match &self {
            EventType::StartScheduling => -1,
            EventType::RequestJob { .. } => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Event {
    event_type: EventType,
    time: u64,
}

#[derive(Debug, Clone)]
pub struct EventQueue {
    que: BinaryHeap<Reverse<Event>>,
}

impl Event {
    pub fn request_job(request_time: u64, job_id: JobID) -> Self {
        let event_type = EventType::RequestJob { job_id };
        Self {
            event_type,
            time: request_time,
        }
    }

    pub fn start_scheduling(time: u64) -> Self {
        Self {
            event_type: EventType::StartScheduling,
            time,
        }
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn event_time(&self) -> u64 {
        self.time
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.time, -self.event_type.priority()).cmp(&(other.time, -other.event_type.priority()))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            que: BinaryHeap::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.que.push(Reverse(event));
    }

    pub fn peek(&self) -> Option<&Event> {
        self.que.peek().map(|Reverse(e)| e)
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.que.pop().map(|Reverse(e)| e)
    }

    pub fn next_event_time(&self) -> Option<u64> {
        self.que.peek().map(|Reverse(e)| e.event_time())
    }

    pub fn is_empty(&self) -> bool {
        self.que.is_empty()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::event::{Event, EventQueue};

    #[test]
    fn test_event_que_add() {
        let mut event_que = EventQueue::new();

        event_que.add_event(Event::request_job(0, 0));
        event_que.add_event(Event::start_scheduling(1));
        event_que.add_event(Event::request_job(1, 1));

        assert_eq!(event_que.next_event_time(), Some(0));
        assert_eq!(event_que.pop().unwrap(), Event::request_job(0, 0));
        assert_eq!(event_que.next_event_time(), Some(1));
        assert_eq!(event_que.pop().unwrap(), Event::request_job(1, 1));
        assert_eq!(event_que.next_event_time(), Some(1));
        assert_eq!(event_que.pop().unwrap(), Event::start_scheduling(1));
    }
}
