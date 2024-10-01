use super::subscriber::{Event, LogRecord};

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum LogState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Logs {
    pub state: LogState,
    pub logs: Vec<LogRecord>,
}

impl Default for Logs {
    fn default() -> Self {
      Logs {
        state: LogState::Disconnected,
        logs: Vec::new(),
      }
    }
}

impl Logs {
  pub fn handle_event(&mut self, event: &Event) {
    match event {
        Event::Connected => {
            self.state = LogState::Connected;
        },
        Event::Disconnected => {
            self.state = LogState::Error(format!("Got disconnected"));
            self.logs.clear();
        },
        Event::Message(message) => {
            self.logs.push(message.clone());
        },
    };
  }
}
