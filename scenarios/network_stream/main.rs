use picoring::PicoByteStream;
use std::io::Write;

fn main() {
    let mut stream = PicoByteStream::new(65536).unwrap();

    // Imagine a large JSON message arrives in two network packets
    // and the first packet ends exactly at the buffer's physical wrap point.
    let packet_1 = b"{\"id\": 123, \"data\": \"";
    let packet_2 = b"very large content\"}";

    stream.write_all(packet_1).unwrap();
    stream.write_all(packet_2).unwrap();

    // TRADITIONAL way: You'd have to check if JSON is split and copy it to a String.
    // PICORING way: Just get the slice and parse.
    let full_data = stream.as_read_slice();

    // Pass directly to a JSON parser (simulated)
    parse_json(full_data);

    println!("Successfully parsed a split network packet without reassembly.");
}

fn parse_json(data: &[u8]) {
    let s = std::str::from_utf8(data).unwrap();
    assert!(s.contains("very large content"));
}
