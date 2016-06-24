use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    Command::new("protoc")
        .args(&["--rust_out", "src/messages/", "src/messages/protobufs.proto"])
        .status()
        .unwrap();
    Command::new("protoc")
        .args(&["--rust_out", "src/client/messages/", "src/client/messages/protobufs.proto"])
        .status()
        .unwrap();
}
