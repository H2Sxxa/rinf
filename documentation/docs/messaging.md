# Messaging

!!! warning

    If you are using Rinf version 5 or earlier, please refer to the [historical documentation](https://github.com/cunarist/rinf/blob/v5.4.0/documentation/docs/writing-code.md). With the introduction of Rinf version 6, a simpler way for communication between Dart and Rust has been implemented, and the system has undergone significant changes.

Special comments empower Protobuf messages with the capability to transmit signals across Dart and Rust. This is achieved by allowing Rinf's code generator to create the necessary channels for communication between Dart and Rust.

There are 2 types of special comments that you can mark messages with.

## 📭 Dart Signal

`[RINF:DART-SIGNAL]` generates a channel from Dart to Rust.

```proto title="Protobuf"
// [RINF:DART-SIGNAL]
message MyDataInput { ... }
```

```dart title="Dart"
MyDataInput( ... ).sendSignalToRust(null);
```

```rust title="Rust"
let mut receiver = MyDataInput::get_dart_signal_receiver();
while let Some(dart_signal) = receiver.recv().await {
    // Custom Rust logic here
}
```

## 📢 Rust Signal

`[RINF:RUST-SIGNAL]` generates a channel from Rust to Dart.

```proto title="Protobuf"
// [RINF:RUST-SIGNAL]
message MyDataOutput { ... }
```

```dart title="Dart"
MyDataOutput.rustSignalStream.listen((rustSignal) {
    // Custom Dart logic here
})
```

```rust title="Rust"
MyDataOutput { ... }.send_signal_to_dart(None);
```

## 🗄️ Binary Data

You can provide binary data as an argument to the `sendSignalToRust()` or `send_signal_to_dart()` method.

Its type should be `Uint8List?` in Dart and `Option<Vec<u8>>` in Rust. Passing binary data separately is recommended over embedding it inside the Protobuf message for better performance, as it avoids the overhead of serialization.
