#![allow(dead_code)]

use flutter_rust_bridge::StreamSink;
use flutter_rust_bridge::SyncReturn;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Clone)]
pub struct RustSignal {
    pub address: String,
    pub bytes: Vec<u8>,
}

pub struct RustRequest {
    pub address: String,
    pub operation: Operation,
    pub bytes: Vec<u8>,
}

pub struct RustRequestUnique {
    pub id: i32,
    pub request: RustRequest,
}

#[derive(Clone)]
pub struct RustResponse {
    pub successful: bool,
    pub bytes: Vec<u8>,
}

impl Default for RustResponse {
    /// Empty response with the successful value of false.
    fn default() -> RustResponse {
        RustResponse {
            successful: false,
            bytes: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct RustResponseUnique {
    pub id: i32,
    pub response: RustResponse,
}

type Cell<T> = RefCell<Option<T>>;
type SharedCell<T> = Arc<Mutex<Cell<T>>>;

type RustSignalStream = StreamSink<RustSignal>;
type RustResponseStream = StreamSink<RustResponseUnique>;
type RustRequestSender = Sender<RustRequestUnique>;
type RustRequestReceiver = Receiver<RustRequestUnique>;

// For thread 0 running Dart
thread_local! {
    pub static REQUEST_SENDER: Cell<RustRequestSender> = RefCell::new(None);
}

// For thread 1~N running Rust
thread_local! {
    pub static SIGNAL_STREAM: Cell<RustSignalStream> = RefCell::new(None);
    pub static RESPONSE_STREAM: Cell<RustResponseStream> = RefCell::new(None);
    pub static RESPONSE_SENDER: Cell<RustResponseUnique> = RefCell::new(None);
}

// For sharing between threads
lazy_static! {
    pub static ref SIGNAL_STREAM_SHARED: SharedCell<RustSignalStream> =
        Arc::new(Mutex::new(RefCell::new(None)));
    pub static ref RESPONSE_STREAM_SHARED: SharedCell<RustResponseStream> =
        Arc::new(Mutex::new(RefCell::new(None)));
    pub static ref REQUST_RECEIVER_SHARED: SharedCell<RustRequestReceiver> =
        Arc::new(Mutex::new(RefCell::new(None)));
}

pub fn prepare_rust_signal_stream(signal_stream: StreamSink<RustSignal>) {
    // Thread 1 running Rust
    let cell = SIGNAL_STREAM_SHARED.lock().unwrap();
    cell.replace(Some(signal_stream));
}

pub fn prepare_rust_response_stream(response_stream: StreamSink<RustResponseUnique>) {
    // Thread 1 running Rust
    let cell = RESPONSE_STREAM_SHARED.lock().unwrap();
    cell.replace(Some(response_stream));
}

pub fn prepare_channels() -> SyncReturn<()> {
    // Thread 0 running Dart
    let (request_sender, request_receiver) = channel(1024);
    REQUEST_SENDER.with(move |inner| {
        inner.replace(Some(request_sender));
    });
    let cell = REQUST_RECEIVER_SHARED.lock().unwrap();
    cell.replace(Some(request_receiver));
    SyncReturn(())
}

pub fn start_rust_logic() {
    // Thread 1 running Rust
    crate::main();
}

pub fn request_to_rust(request_unique: RustRequestUnique) -> SyncReturn<()> {
    // Thread 0 running Dart
    REQUEST_SENDER.with(move |inner| {
        let borrowed = inner.borrow();
        let sender = borrowed.as_ref().unwrap();
        sender.try_send(request_unique).ok();
    });
    SyncReturn(())
}