use std::process::Command;

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
