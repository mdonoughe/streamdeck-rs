use crate::{LogMessagePayload, MessageOut};
use futures::channel::mpsc;
use slog::{Drain, Key, OwnedKVList, Record, KV};
use std::fmt::{self, Write};
use std::sync::Mutex;

pub struct StreamDeckDrain<G, S, M> {
    sink: Mutex<mpsc::UnboundedSender<MessageOut<G, S, M>>>,
}

impl<G, S, M> StreamDeckDrain<G, S, M> {
    pub fn new(sink: mpsc::UnboundedSender<MessageOut<G, S, M>>) -> Self {
        Self {
            sink: Mutex::new(sink),
        }
    }
}

impl<G, S, M> Drain for StreamDeckDrain<G, S, M> {
    type Ok = ();
    type Err = mpsc::TrySendError<MessageOut<G, S, M>>;

    fn log(&self, record: &Record, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let mut message = format!("{} {}", record.level().as_short_str(), record.msg());

        let mut serializer = Serializer { stack: Vec::new() };
        record.kv().serialize(record, &mut serializer).unwrap();
        values.serialize(record, &mut serializer).unwrap();

        let kv_len = serializer.stack.iter().fold(0, |a, b| a + b.len() + 2);
        message.reserve_exact(kv_len);
        while let Some(value) = serializer.stack.pop() {
            write!(message, ", {}", value).unwrap()
        }

        self.sink
            .lock()
            .unwrap()
            .unbounded_send(MessageOut::LogMessage {
                payload: LogMessagePayload { message },
            })
    }
}

struct Serializer {
    stack: Vec<String>,
}

impl slog::Serializer for Serializer {
    fn emit_none(&mut self, key: Key) -> slog::Result {
        self.stack.push(format!("{}: None", key));
        Ok(())
    }
    fn emit_unit(&mut self, key: Key) -> slog::Result {
        self.stack.push(format!("{}: ()", key));
        Ok(())
    }
    fn emit_bool(&mut self, key: Key, val: bool) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_u8(&mut self, key: Key, val: u8) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_i8(&mut self, key: Key, val: i8) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_u16(&mut self, key: Key, val: u16) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_i16(&mut self, key: Key, val: i16) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_u32(&mut self, key: Key, val: u32) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_i32(&mut self, key: Key, val: i32) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_f32(&mut self, key: Key, val: f32) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_i64(&mut self, key: Key, val: i64) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_f64(&mut self, key: Key, val: f64) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_str(&mut self, key: Key, val: &str) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> slog::Result {
        self.stack.push(format!("{}: {}", key, val));
        Ok(())
    }
}
