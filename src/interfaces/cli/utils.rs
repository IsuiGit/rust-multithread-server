use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Name
    #[arg(long)]
    name: String,
    /// Host
    #[arg(long)]
    host: String,
    /// Port
    #[arg(long)]
    port: u16,
    /// Log file path (optional, defaults to "app.log")
    #[arg(long, default_value = "app.log")]
    log_file_path: String,
}

pub fn parse() -> Result<(String, [u8; 4], u16, String), String> {
    // Load parser
    let _args = Args::parse();
    // Get host as string
    let host: Vec<&str> = _args.host.split('.').collect();
    // Check host
    if host.len() != 4{
        return Err("Host must be an octets splieted by dots like 0.0.0.0".to_string());
    }
    // Check port on zero value and system binding ports
    if _args.port == 0{
        return Err("Port mustn't be zero".to_string());
    }
    if _args.port == 8080 || _args.port == 80 {
        return Err("Cann't bind system port".to_string());
    }
    // Parse host as vec of u8
    let mut octets = [0u8; 4];
    for (i, part) in host.iter().enumerate(){
        match part.parse::<u8>(){
            Ok(num) => octets[i] = num,
            Err(_) => return Err(format!("Uncorrect part of host: {}", part))
        }
    }
    // return octets vec and port
    Ok((_args.name, octets, _args.port, _args.log_file_path))
}
