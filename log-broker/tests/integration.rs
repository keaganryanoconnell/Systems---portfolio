use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use log_broker::{BrokerClient, LogBroker, SegmentConfig};

#[test]
fn test_broker_start_stop() {
    let dir = tempfile::tempdir().unwrap();
    let config = SegmentConfig::default();
    let broker = LogBroker::new(dir.path(), config).unwrap();

    let bind_addr = "127.0.0.1:19991";

    let handle = thread::spawn(move || {
        let _ = broker.start(bind_addr);
    });

    thread::sleep(Duration::from_millis(200));

    let stream = TcpStream::connect("127.0.0.1:19991");
    assert!(stream.is_ok(), "should connect to broker");

    drop(handle);
}

#[test]
fn test_produce_and_consume() {
    let dir = tempfile::tempdir().unwrap();
    let config = SegmentConfig::default();
    let broker = LogBroker::new(dir.path(), config).unwrap();

    let bind_addr = "127.0.0.1:19992";

    let handle = thread::spawn(move || {
        let _ = broker.start(bind_addr);
    });

    thread::sleep(Duration::from_millis(200));

    let mut client = BrokerClient::connect("127.0.0.1:19992").unwrap();

    let offset = client.produce("test-topic", b"key-1", b"value-1").unwrap();
    assert_eq!(offset, 0);

    let msg = client
        .produce("test-topic", b"key-2", b"second-value")
        .unwrap();
    assert_eq!(msg, 1);

    let messages = client.fetch("test-topic", 0, 1024 * 1024).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].0, 0);
    assert_eq!(&messages[0].1, b"key-1");
    assert_eq!(&messages[0].2, b"value-1");
    assert_eq!(messages[1].0, 1);
    assert_eq!(&messages[1].1, b"key-2");
    assert_eq!(&messages[1].2, b"second-value");

    let (earliest, latest) = client.list_offsets("test-topic").unwrap();
    assert_eq!(earliest, 0);
    assert_eq!(latest, 2);

    drop(handle);
}

#[test]
fn test_fetch_nonexistent_topic() {
    let dir = tempfile::tempdir().unwrap();
    let config = SegmentConfig::default();
    let broker = LogBroker::new(dir.path(), config).unwrap();

    let bind_addr = "127.0.0.1:19993";

    let handle = thread::spawn(move || {
        let _ = broker.start(bind_addr);
    });

    thread::sleep(Duration::from_millis(200));

    let mut client = BrokerClient::connect("127.0.0.1:19993").unwrap();

    let (earliest, latest) = client.list_offsets("nonexistent").unwrap();
    assert_eq!(earliest, 0);
    assert_eq!(latest, 0);

    drop(handle);
}

#[test]
fn test_multiple_topics() {
    let dir = tempfile::tempdir().unwrap();
    let config = SegmentConfig::default();
    let broker = LogBroker::new(dir.path(), config).unwrap();

    let bind_addr = "127.0.0.1:19994";

    let handle = thread::spawn(move || {
        let _ = broker.start(bind_addr);
    });

    thread::sleep(Duration::from_millis(200));

    let mut client = BrokerClient::connect("127.0.0.1:19994").unwrap();

    client.produce("orders", b"o1", b"buy-100").unwrap();
    client.produce("orders", b"o2", b"sell-50").unwrap();
    client.produce("users", b"u1", b"alice").unwrap();

    let orders = client.fetch("orders", 0, 1024 * 1024).unwrap();
    assert_eq!(orders.len(), 2);

    let users = client.fetch("users", 0, 1024 * 1024).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(&users[0].2, b"alice");

    drop(handle);
}
