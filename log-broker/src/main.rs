use std::env;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use log_broker::{BrokerClient, BrokerError, LogBroker, SegmentConfig};

fn print_usage() {
    eprintln!("log-broker — Real-time distributed log and pub-sub broker");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  log-broker start   --data-dir <path> [--bind <addr>]");
    eprintln!("  log-broker produce --broker <addr> --topic <name> [--key <k>] [--value <v>] [--file <path>]");
    eprintln!("  log-broker consume --broker <addr> --topic <name> [--client-id <id>]");
    eprintln!("  log-broker offsets --broker <addr> --topic <name>");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    let result = match command.as_str() {
        "start" => cmd_start(&args),
        "produce" => cmd_produce(&args),
        "consume" => cmd_consume(&args),
        "offsets" => cmd_offsets(&args),
        "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("error: unknown command '{}'", command);
            print_usage();
            std::process::exit(1);
        }
    };

    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
    }
    None
}

fn require_arg(args: &[String], flag: &str) -> Result<String, BrokerError> {
    get_arg(args, flag)
        .ok_or_else(|| BrokerError::InvalidArgument(format!("missing required flag: {}", flag)))
}

fn cmd_start(args: &[String]) -> Result<(), BrokerError> {
    let data_dir = require_arg(args, "--data-dir")?;
    let bind_addr = get_arg(args, "--bind").unwrap_or_else(|| "127.0.0.1:9092".to_string());

    let config = SegmentConfig::default();
    let broker = LogBroker::new(&PathBuf::from(&data_dir), config)?;

    eprintln!("[log-broker] data directory: {}", data_dir);
    broker.start(&bind_addr)
}

fn cmd_produce(args: &[String]) -> Result<(), BrokerError> {
    let broker_addr = require_arg(args, "--broker")?;
    let topic = require_arg(args, "--topic")?;
    let key = get_arg(args, "--key").unwrap_or_default();
    let value = get_arg(args, "--value").unwrap_or_default();
    let file_path = get_arg(args, "--file");

    let (key_bytes, value_bytes) = if let Some(ref path) = file_path {
        let mut file = std::fs::File::open(path).map_err(BrokerError::from)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(BrokerError::from)?;
        (key.as_bytes().to_vec(), contents)
    } else {
        if value.is_empty() && key.is_empty() {
            let mut stdin_data = String::new();
            io::stdin()
                .read_to_string(&mut stdin_data)
                .map_err(BrokerError::from)?;
            (b"stdin".to_vec(), stdin_data.into_bytes())
        } else {
            (key.into_bytes(), value.into_bytes())
        }
    };

    let mut client = BrokerClient::connect(&broker_addr)?;
    let offset = client.produce(&topic, &key_bytes, &value_bytes)?;

    println!("produced to topic '{}' at offset {}", topic, offset);
    Ok(())
}

fn cmd_consume(args: &[String]) -> Result<(), BrokerError> {
    let broker_addr = require_arg(args, "--broker")?;
    let topic = require_arg(args, "--topic")?;
    let _client_id = get_arg(args, "--client-id").unwrap_or_else(|| "consumer-1".to_string());

    let mut client = BrokerClient::connect(&broker_addr)?;

    let (earliest, latest) = client.list_offsets(&topic)?;
    if earliest == latest {
        eprintln!("topic '{}' is empty (no messages yet)", topic);
        return Ok(());
    }

    eprintln!(
        "consuming topic '{}' (offsets: {} → {})",
        topic, earliest, latest
    );

    let messages = client.fetch(&topic, earliest, 1024 * 1024)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for (offset, key, value) in &messages {
        let key_str = String::from_utf8_lossy(key);
        let value_str = String::from_utf8_lossy(value);
        writeln!(
            handle,
            "[offset {}] key={} value={}",
            offset, key_str, value_str
        )
        .map_err(BrokerError::from)?;
    }

    eprintln!("consumed {} messages", messages.len());
    Ok(())
}

fn cmd_offsets(args: &[String]) -> Result<(), BrokerError> {
    let broker_addr = require_arg(args, "--broker")?;
    let topic = require_arg(args, "--topic")?;

    let mut client = BrokerClient::connect(&broker_addr)?;
    let (earliest, latest) = client.list_offsets(&topic)?;

    println!("topic: {}", topic);
    println!("  earliest offset: {}", earliest);
    println!("  latest  offset: {}", latest);
    println!("  messages:       {}", latest.saturating_sub(earliest));

    Ok(())
}
