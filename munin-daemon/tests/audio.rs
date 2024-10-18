use std::io::Cursor;

use rodio::{Decoder, OutputStream, Sink};

#[test]
fn test_audio() {
    // Open a stream to the default output device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Create a new Sink to play the audio
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Embed a small wav audio file in the binary (replace with your file)
    let audio_data = include_bytes!("../assets/rickroll.mp3");

    // Decode the audio file from the embedded bytes
    let cursor = Cursor::new(audio_data);
    let source = Decoder::new(cursor).unwrap();

    // Append the decoded audio to the sink
    sink.append(source);

    // Sleep until the sound finishes playing
    sink.sleep_until_end();
}
