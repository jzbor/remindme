#[cfg(feature = "server")]
use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use crate::time::{self, Time};

#[cfg(feature = "server")]
pub type ReminderSet = HashSet<Reminder>;
pub type ReminderId = u64;


#[derive(Debug,Clone,Eq,PartialEq,Serialize,Deserialize,Hash)]
pub struct Reminder {
    #[serde(serialize_with = "time::serialize")]
    #[serde(deserialize_with = "time::deserialize")]
    date: Time,
    id: u64,
    message: String,
    state: ReminderState,
}

#[derive(Debug,Copy,Clone,Hash,Serialize,Deserialize,Eq,PartialEq)]
pub enum ReminderState {
    Upcoming,
    Pending,
    Expired,
}

#[derive(Serialize, Deserialize)]
pub struct ReminderRequest {
    #[serde(serialize_with = "time::serialize")]
    #[serde(deserialize_with = "time::deserialize")]
    date: Time,
    message: String,
}

impl ReminderRequest {
    #[cfg(feature = "client")]
    pub fn new(date: Time, message: String) -> Self {
        ReminderRequest { date, message, }
    }
}

impl Reminder {
    #[cfg(feature = "server")]
    pub fn new(request: ReminderRequest, id: ReminderId) -> Self {
        Reminder {
            date: request.date,
            message: request.message,
            id,
            state: ReminderState::Upcoming,
        }
    }

    #[cfg(feature = "server")]
    pub fn expired(&self) -> bool {
        self.date < time::now()
    }

    #[cfg(feature = "server")]
    pub fn set_state(&mut self, state: ReminderState) {
        self.state = state;
    }

    pub fn id(&self) -> ReminderId {
        self.id
    }

    #[cfg(feature = "client")]
    pub fn date(&self) -> Time {
        self.date
    }

    #[cfg(feature = "client")]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[cfg(feature = "client")]
    pub fn state(&self) -> ReminderState {
        self.state
    }
}
