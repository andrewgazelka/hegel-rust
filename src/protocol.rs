//! Binary protocol implementation for Hegel.
//!
//! This module implements the binary packet protocol for communicating with
//! the Hegel server. The protocol uses:
//! - 20-byte binary headers with magic number and CRC32 checksum
//! - CBOR-encoded payloads
//! - Channel multiplexing for concurrent operations

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use ciborium::Value as CborValue;

// Protocol constants
const MAGIC: u32 = 0x4845474C; // "HEGL" in big-endian
const HEADER_SIZE: usize = 20;
const REPLY_BIT: u32 = 1 << 31;
const TERMINATOR: u8 = 0x0A;

/// Version negotiation message sent by client
pub const VERSION_NEGOTIATION_MESSAGE: &[u8] = b"Hegel/1.0";
/// Expected response for successful version negotiation
pub const VERSION_NEGOTIATION_OK: &[u8] = b"Ok";

/// A packet in the wire protocol.
#[derive(Debug, Clone)]
pub struct Packet {
    pub channel: u32,
    pub message_id: u32,
    pub is_reply: bool,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Create a new request packet.
    pub fn request(channel: u32, message_id: u32, payload: Vec<u8>) -> Self {
        Self {
            channel,
            message_id,
            is_reply: false,
            payload,
        }
    }

    /// Create a new reply packet.
    pub fn reply(channel: u32, message_id: u32, payload: Vec<u8>) -> Self {
        Self {
            channel,
            message_id,
            is_reply: true,
            payload,
        }
    }
}

/// Write a packet to a stream.
pub fn write_packet<W: Write>(writer: &mut W, packet: &Packet) -> std::io::Result<()> {
    let message_id = if packet.is_reply {
        packet.message_id | REPLY_BIT
    } else {
        packet.message_id
    };

    // Build header
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_be_bytes());
    // Checksum placeholder at 4..8, filled after
    header[8..12].copy_from_slice(&packet.channel.to_be_bytes());
    header[12..16].copy_from_slice(&message_id.to_be_bytes());
    header[16..20].copy_from_slice(&(packet.payload.len() as u32).to_be_bytes());

    // Calculate checksum over header (with checksum field as zeros) + payload
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&header);
    hasher.update(&packet.payload);
    let checksum = hasher.finalize();
    header[4..8].copy_from_slice(&checksum.to_be_bytes());

    // Write header + payload + terminator
    writer.write_all(&header)?;
    writer.write_all(&packet.payload)?;
    writer.write_all(&[TERMINATOR])?;
    writer.flush()?;

    Ok(())
}

/// Read a packet from a stream.
pub fn read_packet<R: Read>(reader: &mut R) -> std::io::Result<Packet> {
    // Read header
    let mut header = [0u8; HEADER_SIZE];
    reader.read_exact(&mut header)?;

    let magic = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let checksum = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    let channel = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
    let message_id_raw = u32::from_be_bytes([header[12], header[13], header[14], header[15]]);
    let length = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);

    // Validate magic
    if magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid magic number: expected 0x{:08X}, got 0x{:08X}", MAGIC, magic),
        ));
    }

    // Extract reply bit
    let is_reply = message_id_raw & REPLY_BIT != 0;
    let message_id = message_id_raw & !REPLY_BIT;

    // Read payload
    let mut payload = vec![0u8; length as usize];
    reader.read_exact(&mut payload)?;

    // Read terminator
    let mut terminator = [0u8; 1];
    reader.read_exact(&mut terminator)?;
    if terminator[0] != TERMINATOR {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid terminator: expected 0x{:02X}, got 0x{:02X}", TERMINATOR, terminator[0]),
        ));
    }

    // Verify checksum
    let mut header_for_check = header;
    header_for_check[4..8].copy_from_slice(&[0, 0, 0, 0]); // Zero out checksum field
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&header_for_check);
    hasher.update(&payload);
    let computed_checksum = hasher.finalize();
    if computed_checksum != checksum {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Checksum mismatch: expected 0x{:08X}, got 0x{:08X}", checksum, computed_checksum),
        ));
    }

    Ok(Packet {
        channel,
        message_id,
        is_reply,
        payload,
    })
}

/// A logical channel on a connection.
pub struct Channel {
    pub channel_id: u32,
    connection: Arc<Connection>,
    next_message_id: AtomicU32,
    /// Inbox for received packets (responses and requests)
    responses: Mutex<HashMap<u32, Vec<u8>>>,
    requests: Mutex<VecDeque<Packet>>,
}

impl Channel {
    fn new(channel_id: u32, connection: Arc<Connection>) -> Self {
        Self {
            channel_id,
            connection,
            next_message_id: AtomicU32::new(1),
            responses: Mutex::new(HashMap::new()),
            requests: Mutex::new(VecDeque::new()),
        }
    }

    /// Create a clone of this channel for embedded mode.
    /// This creates a new Channel with the same channel_id but separate state.
    pub fn clone_for_embedded(&self) -> Self {
        Self {
            channel_id: self.channel_id,
            connection: Arc::clone(&self.connection),
            next_message_id: AtomicU32::new(1),
            responses: Mutex::new(HashMap::new()),
            requests: Mutex::new(VecDeque::new()),
        }
    }

    /// Send a request and return the message ID.
    pub fn send_request(&self, payload: Vec<u8>) -> std::io::Result<u32> {
        let message_id = self.next_message_id.fetch_add(1, Ordering::SeqCst);
        let packet = Packet::request(self.channel_id, message_id, payload);
        self.connection.send_packet(&packet)?;
        Ok(message_id)
    }

    /// Send a response to a request.
    pub fn send_response(&self, message_id: u32, payload: Vec<u8>) -> std::io::Result<()> {
        let packet = Packet::reply(self.channel_id, message_id, payload);
        self.connection.send_packet(&packet)
    }

    /// Wait for a response to a previously sent request.
    pub fn receive_response(&self, message_id: u32) -> std::io::Result<Vec<u8>> {
        loop {
            // Check if we already have the response
            {
                let mut responses = self.responses.lock().unwrap();
                if let Some(payload) = responses.remove(&message_id) {
                    return Ok(payload);
                }
            }

            // Process one message from the connection
            self.process_one_message()?;
        }
    }

    /// Wait for an incoming request.
    pub fn receive_request(&self) -> std::io::Result<(u32, Vec<u8>)> {
        loop {
            // Check if we already have a request
            {
                let mut requests = self.requests.lock().unwrap();
                if let Some(packet) = requests.pop_front() {
                    return Ok((packet.message_id, packet.payload));
                }
            }

            // Process one message from the connection
            self.process_one_message()?;
        }
    }

    /// Process one incoming message and route it appropriately.
    fn process_one_message(&self) -> std::io::Result<()> {
        let packet = self.connection.receive_packet_for_channel(self.channel_id)?;

        if packet.is_reply {
            let mut responses = self.responses.lock().unwrap();
            responses.insert(packet.message_id, packet.payload);
        } else {
            let mut requests = self.requests.lock().unwrap();
            requests.push_back(packet);
        }

        Ok(())
    }

    /// Send a CBOR request and wait for the CBOR response.
    pub fn request(&self, message: &CborValue) -> std::io::Result<CborValue> {
        let mut payload = Vec::new();
        ciborium::into_writer(message, &mut payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let id = self.send_request(payload)?;
        let response_bytes = self.receive_response(id)?;

        let response: CborValue = ciborium::from_reader(&response_bytes[..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Check for error response
        if let CborValue::Map(ref map) = response {
            for (k, v) in map {
                if let CborValue::Text(key) = k {
                    if key == "error" {
                        let error_msg = match v {
                            CborValue::Text(s) => s.clone(),
                            _ => format!("{:?}", v),
                        };
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Server error: {}", error_msg),
                        ));
                    }
                }
            }
            // Extract result field
            for (k, v) in map {
                if let CborValue::Text(key) = k {
                    if key == "result" {
                        return Ok(v.clone());
                    }
                }
            }
        }

        Ok(response)
    }
}

/// A connection to the Hegel server.
pub struct Connection {
    stream: Mutex<UnixStream>,
    /// Packets that arrived for channels other than the one being processed
    pending_packets: Mutex<HashMap<u32, VecDeque<Packet>>>,
    next_channel_id: AtomicU32,
    channels: Mutex<HashMap<u32, ()>>, // Track which channels exist
}

impl Connection {
    /// Create a new connection from a Unix stream.
    pub fn new(stream: UnixStream) -> Arc<Self> {
        Arc::new(Self {
            stream: Mutex::new(stream),
            pending_packets: Mutex::new(HashMap::new()),
            next_channel_id: AtomicU32::new(1), // 0 is reserved for control
            channels: Mutex::new(HashMap::new()),
        })
    }

    /// Get the control channel (channel 0).
    pub fn control_channel(self: &Arc<Self>) -> Channel {
        Channel::new(0, Arc::clone(self))
    }

    /// Create a new channel.
    pub fn new_channel(self: &Arc<Self>) -> Channel {
        let channel_id = self.next_channel_id.fetch_add(1, Ordering::SeqCst);
        self.channels.lock().unwrap().insert(channel_id, ());
        Channel::new(channel_id, Arc::clone(self))
    }

    /// Connect to an existing channel (created by the other side).
    pub fn connect_channel(self: &Arc<Self>, channel_id: u32) -> Channel {
        self.channels.lock().unwrap().insert(channel_id, ());
        Channel::new(channel_id, Arc::clone(self))
    }

    /// Send a packet.
    pub fn send_packet(&self, packet: &Packet) -> std::io::Result<()> {
        let mut stream = self.stream.lock().unwrap();
        write_packet(&mut *stream, packet)
    }

    /// Receive a packet for a specific channel.
    /// If a packet for a different channel arrives, it's queued for later.
    pub fn receive_packet_for_channel(&self, channel_id: u32) -> std::io::Result<Packet> {
        // First check pending packets
        {
            let mut pending = self.pending_packets.lock().unwrap();
            if let Some(queue) = pending.get_mut(&channel_id) {
                if let Some(packet) = queue.pop_front() {
                    return Ok(packet);
                }
            }
        }

        // Read from stream until we get a packet for our channel
        loop {
            let packet = {
                let mut stream = self.stream.lock().unwrap();
                read_packet(&mut *stream)?
            };

            if packet.channel == channel_id {
                return Ok(packet);
            }

            // Queue for another channel
            let mut pending = self.pending_packets.lock().unwrap();
            pending.entry(packet.channel).or_default().push_back(packet);
        }
    }

    /// Close the connection.
    pub fn close(&self) -> std::io::Result<()> {
        let stream = self.stream.lock().unwrap();
        stream.shutdown(std::net::Shutdown::Both)
    }
}

/// Perform version negotiation on a connection.
pub fn negotiate_version(connection: &Arc<Connection>) -> std::io::Result<()> {
    let control = connection.control_channel();
    let id = control.send_request(VERSION_NEGOTIATION_MESSAGE.to_vec())?;
    let response = control.receive_response(id)?;

    if response == VERSION_NEGOTIATION_OK {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Version negotiation failed: {:?}", String::from_utf8_lossy(&response)),
        ))
    }
}

/// Helper to convert serde_json::Value to ciborium::Value.
pub fn json_to_cbor(json: &serde_json::Value) -> CborValue {
    match json {
        serde_json::Value::Null => CborValue::Null,
        serde_json::Value::Bool(b) => CborValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CborValue::Integer(i.into())
            } else if let Some(u) = n.as_u64() {
                CborValue::Integer(u.into())
            } else if let Some(f) = n.as_f64() {
                CborValue::Float(f)
            } else {
                CborValue::Null
            }
        }
        serde_json::Value::String(s) => CborValue::Text(s.clone()),
        serde_json::Value::Array(arr) => {
            CborValue::Array(arr.iter().map(json_to_cbor).collect())
        }
        serde_json::Value::Object(obj) => {
            CborValue::Map(
                obj.iter()
                    .map(|(k, v)| (CborValue::Text(k.clone()), json_to_cbor(v)))
                    .collect(),
            )
        }
    }
}

/// Helper to convert ciborium::Value to serde_json::Value.
pub fn cbor_to_json(cbor: &CborValue) -> serde_json::Value {
    match cbor {
        CborValue::Null => serde_json::Value::Null,
        CborValue::Bool(b) => serde_json::Value::Bool(*b),
        CborValue::Integer(i) => {
            let n: i128 = (*i).into();
            if let Ok(i) = i64::try_from(n) {
                serde_json::Value::Number(i.into())
            } else if let Ok(u) = u64::try_from(n) {
                serde_json::Value::Number(u.into())
            } else {
                // Fallback for very large integers
                serde_json::Value::String(n.to_string())
            }
        }
        CborValue::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        CborValue::Text(s) => serde_json::Value::String(s.clone()),
        CborValue::Bytes(b) => {
            // Encode bytes as base64
            let mut encoded = Vec::new();
            base64_encode(b, &mut encoded);
            serde_json::Value::String(String::from_utf8_lossy(&encoded).into_owned())
        }
        CborValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(cbor_to_json).collect())
        }
        CborValue::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    if let CborValue::Text(key) = k {
                        Some((key.clone(), cbor_to_json(v)))
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        CborValue::Tag(_, inner) => cbor_to_json(inner),
        _ => serde_json::Value::Null,
    }
}

/// Simple base64 encoding.
fn base64_encode(data: &[u8], output: &mut Vec<u8>) {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        output.push(ALPHABET[b0 >> 2]);
        output.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)]);

        if chunk.len() > 1 {
            output.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)]);
        } else {
            output.push(b'=');
        }

        if chunk.len() > 2 {
            output.push(ALPHABET[b2 & 0x3f]);
        } else {
            output.push(b'=');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test_packet_roundtrip() {
        let (mut client, mut server) = UnixStream::pair().unwrap();

        let packet = Packet::request(1, 42, b"hello world".to_vec());
        write_packet(&mut client, &packet).unwrap();

        let received = read_packet(&mut server).unwrap();
        assert_eq!(received.channel, 1);
        assert_eq!(received.message_id, 42);
        assert!(!received.is_reply);
        assert_eq!(received.payload, b"hello world");
    }

    #[test]
    fn test_reply_packet() {
        let (mut client, mut server) = UnixStream::pair().unwrap();

        let packet = Packet::reply(2, 100, b"response".to_vec());
        write_packet(&mut client, &packet).unwrap();

        let received = read_packet(&mut server).unwrap();
        assert_eq!(received.channel, 2);
        assert_eq!(received.message_id, 100);
        assert!(received.is_reply);
        assert_eq!(received.payload, b"response");
    }

    #[test]
    fn test_json_cbor_conversion() {
        let json = serde_json::json!({
            "type": "integer",
            "minimum": 0,
            "maximum": 100
        });

        let cbor = json_to_cbor(&json);
        let back = cbor_to_json(&cbor);

        assert_eq!(json, back);
    }
}
