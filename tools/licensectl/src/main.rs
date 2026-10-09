//! Vendor-side licence signing tool.
//!
//! This is deliberately a *separate* crate: it is never built into the app
//! bundle, and the private key it uses must never go near a customer machine.
//! Run it from the repo with:
//!
//! ```text
//! cargo run --manifest-path tools/licensectl/Cargo.toml -- keygen --out .vendor/key.txt
//! cargo run --manifest-path tools/licensectl/Cargo.toml -- issue --key .vendor/key.txt --request @request.txt --out acme.lic
//! ```
//!
//! The matching **public** key is what gets compiled into the app.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ledgerly_license::{
    decode_request, hex, unhex, License, LicenseFile, RequestCode, FORMAT_VERSION,
};
use std::process::ExitCode;

const DEFAULT_KEY_ID: &str = "ledgerly-dev";
const SUPPORTED_FEATURES: &[&str] = &[
    "reports",
    "pdf_templates",
    "branding",
    "scheduled_backups",
    "batch_export",
    "multi_currency",
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        usage();
        return ExitCode::FAILURE;
    };

    let result = match command {
        "keygen" => keygen(&args[1..]),
        "pubkey" => pubkey(&args[1..]),
        "issue" => issue(&args[1..]),
        "request" => request(&args[1..]),
        "verify" => verify(&args[1..]),
        "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        other => Err(format!("unknown command '{other}'\n")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprint!("{message}");
            usage();
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!(
        "\
ledgerly-licensectl — vendor signing tool

  keygen  --out <path>                          create a signing key (keep it secret)
  pubkey  --key <path>                          print the public key to embed in the app
  issue   --key <path> --request <code|@file>   issue a licence
          [--sku ledgerly] [--plan perpetual|term]
          [--expires YYYY-MM-DD] [--features a,b,c]
          [--out <path>]                        (defaults to stdout)
  verify  --file <path> --pubkey <hex>          check a licence file
  request --serial <id> --machine <fp>          build a request code (for testing)

Supported features: {SUPPORTED_FEATURES:?}"
    );
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let index = args.iter().position(|arg| arg == name)?;
    args.get(index + 1).map(String::as_str)
}

fn keygen(args: &[String]) -> Result<(), String> {
    let out = flag(args, "--out").ok_or("keygen needs --out <path>\n")?;

    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret).map_err(|error| format!("no randomness: {error}\n"))?;
    let signing = SigningKey::from_bytes(&secret);

    if let Some(parent) = std::path::Path::new(out).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| format!("{error}\n"))?;
        }
    }
    std::fs::write(out, hex(&secret)).map_err(|error| format!("{error}\n"))?;

    println!("private key written to {out}  (keep it secret; never commit it)");
    println!("public key (embed in the app):");
    println!("{}", hex(&signing.verifying_key().to_bytes()));
    Ok(())
}

fn read_key(path: &str) -> Result<SigningKey, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{error}\n"))?;
    let bytes = unhex(&text).ok_or("the key file is not valid hex\n")?;
    let secret: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "the key file must hold 32 bytes\n")?;
    Ok(SigningKey::from_bytes(&secret))
}

fn pubkey(args: &[String]) -> Result<(), String> {
    let key = flag(args, "--key").ok_or("pubkey needs --key <path>\n")?;
    let signing = read_key(key)?;
    println!("{}", hex(&signing.verifying_key().to_bytes()));
    Ok(())
}

fn read_request(value: &str) -> Result<RequestCode, String> {
    let text = match value.strip_prefix('@') {
        Some(path) => std::fs::read_to_string(path).map_err(|error| format!("{error}\n"))?,
        None => value.to_string(),
    };
    decode_request(&text).ok_or_else(|| {
        "could not read the request code — copy it whole, including every group\n".to_string()
    })
}

fn issue(args: &[String]) -> Result<(), String> {
    let key_path = flag(args, "--key").ok_or("issue needs --key <path>\n")?;
    let request_value = flag(args, "--request").ok_or("issue needs --request <code|@file>\n")?;

    let signing = read_key(key_path)?;
    let request = read_request(request_value)?;

    if request.version > FORMAT_VERSION {
        return Err(format!(
            "that request comes from a newer app (format {})\n",
            request.version
        ));
    }

    let sku = flag(args, "--sku").unwrap_or(&request.sku).to_string();
    let plan = flag(args, "--plan").unwrap_or("perpetual").to_string();
    let expires_at = flag(args, "--expires").map(str::to_string);
    let features: Vec<String> = match flag(args, "--features") {
        Some(list) => list.split(',').map(|f| f.trim().to_string()).collect(),
        None => SUPPORTED_FEATURES
            .iter()
            .map(|f| (*f).to_string())
            .collect(),
    };

    if plan == "perpetual" && expires_at.is_some() {
        return Err("a perpetual licence cannot have --expires\n".to_string());
    }
    if plan == "term" && expires_at.is_none() {
        return Err("a term licence needs --expires YYYY-MM-DD\n".to_string());
    }

    let license = License {
        version: FORMAT_VERSION,
        key_id: flag(args, "--key-id").unwrap_or(DEFAULT_KEY_ID).to_string(),
        sku: sku.clone(),
        plan,
        features,
        issued_at: today(),
        expires_at,
        binding: ledgerly_license::binding_for(&request.app_serial, &request.machine_fp, &sku),
    };

    let signature = signing.sign(&license.signing_bytes());
    let file = LicenseFile {
        signature: hex(&signature.to_bytes()),
        license,
    };

    let json = serde_json::to_string_pretty(&file).map_err(|error| format!("{error}\n"))?;
    match flag(args, "--out") {
        Some(path) => {
            std::fs::write(path, &json).map_err(|error| format!("{error}\n"))?;
            println!("licence written to {path}");
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Build a request code from raw values. Handy for testing the flow without a
/// second machine; the app produces these itself in normal use.
fn request(args: &[String]) -> Result<(), String> {
    let serial = flag(args, "--serial").ok_or("request needs --serial <id>\n")?;
    let machine = flag(args, "--machine").ok_or("request needs --machine <fingerprint>\n")?;
    let sku = flag(args, "--sku").unwrap_or("ledgerly");

    let code = ledgerly_license::encode_request(&RequestCode {
        version: FORMAT_VERSION,
        sku: sku.to_string(),
        app_serial: serial.to_string(),
        machine_fp: machine.to_string(),
        app_version: flag(args, "--app-version").unwrap_or("0.0.0").to_string(),
        os: flag(args, "--os").unwrap_or("unknown").to_string(),
    });

    match flag(args, "--out") {
        Some(path) => {
            std::fs::write(path, &code).map_err(|error| format!("{error}\n"))?;
            println!("request code written to {path}");
        }
        None => println!("{code}"),
    }
    Ok(())
}

fn verify(args: &[String]) -> Result<(), String> {
    let file_path = flag(args, "--file").ok_or("verify needs --file <path>\n")?;
    let public = flag(args, "--pubkey").ok_or("verify needs --pubkey <hex>\n")?;

    let text = std::fs::read_to_string(file_path).map_err(|error| format!("{error}\n"))?;
    let file: LicenseFile = serde_json::from_str(&text).map_err(|error| format!("{error}\n"))?;

    let key_bytes = unhex(public).ok_or("--pubkey must be hex\n")?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "--pubkey must be 32 bytes\n")?;
    let verifying = VerifyingKey::from_bytes(&key).map_err(|error| format!("{error}\n"))?;

    let signature_bytes = unhex(&file.signature).ok_or("the signature is not hex\n")?;
    let signature: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| "the signature must be 64 bytes\n")?;

    verifying
        .verify(
            &file.license.signing_bytes(),
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| "signature is NOT valid\n".to_string())?;

    println!("signature is valid");
    println!("  key id   {}", file.license.key_id);
    println!("  plan     {}", file.license.plan);
    println!("  features {:?}", file.license.features);
    println!("  binding  {}", file.license.binding);
    Ok(())
}

/// Today from the system clock, as YYYY-MM-DD (UTC is fine for an issue date).
fn today() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0) as i64;
    civil_from_days(seconds / 86_400)
}

/// Howard Hinnant's days-from-civil, inverted.
fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as i64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}
