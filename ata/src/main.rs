use clap::Parser;
use hyper::Body;
use hyper::Client;
use hyper::Method;
use hyper::Request;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::result::Result;
use toml::from_str;

mod help;

#[derive(Clone, Deserialize, Debug)]
struct Config {
    api_key: String,
    model: String,
    max_tokens: i64,
    temperature: i64
}

type TokioResult<T, E = Box<dyn Error + Send + Sync>> = Result<T, E>;

#[tokio::main]
async fn prompt_model(config: Config, prompt: String) -> TokioResult<String> {
    let api_key: String = config.clone().api_key;
    let model: String = config.clone().model;
    let max_tokens: i64 = config.clone().max_tokens;
    let temperature: i64 = config.temperature;

    let mut sanitized_input: String = prompt.clone();
    sanitized_input.pop();
    sanitized_input = sanitized_input.replace("\"", "\\\"");
    let bearer = format!("Bearer {}", api_key);
    // Passing newlines behind the prompt to get a more chat-like experience.
    let body = json!({
        "model": model,
        "prompt": format!("{}\\n\\n", sanitized_input),
        "max_tokens": max_tokens,
        "temperature": temperature
    }).to_string();
    // println!("{}", body);

    let req = Request::builder()
        .method(Method::POST)
        .uri("https://api.openai.com/v1/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", bearer)
        .body(Body::from(body))?;

    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;

    // println!("{}", String::from_utf8(body_bytes.clone().to_vec()).unwrap());

    let v: Value = serde_json::from_slice(&body_bytes)?;
    if v.get("error").is_some() {
        let text: String = v["error"]["message"].to_string();
        Ok(text)
    } else {
        let text: String = v["choices"][0]["text"].to_string();
        Ok(text)
    }
}

fn remove_outer_quotation_marks(mut text: String) -> String {
    text.pop();
    text.remove(0);
    text
}

fn remove_leading_newlines(text: String) -> String {
    let re = regex::Regex::new(r"^[\n]*").unwrap();
    re.replace_all(&text, "").into_owned()
}

fn sanitize_response(response: String) -> String {
    let mut text = response;
    text = text.replace("\\n", "\n");
    text = remove_outer_quotation_marks(text);
    text = text.replace("\\\"", "\"");
    remove_leading_newlines(text)
}

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn ata_message(args: Vec<String>) {
    panic!(
        "Usage: `{} --config=<Path to ata.toml>` or have ata.toml in the current dir.",
        args[0]
    )
}

/// Ask the Terminal Anything (ATA): OpenAI GPT in the terminal.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Flags {
    /// Path to the configuration TOML file.
    #[arg(short = 'c', long = "config", default_value = "ata.toml")]
    config: String,

    /// Print the keyboard shortcuts.
    #[arg(long)]
    print_shortcuts: bool,
}

fn main() -> TokioResult<()> {
    let args: Vec<String> = env::args().collect();
    let flags: Flags = Flags::parse();
    if flags.print_shortcuts {
        help::commands();
        return Ok(());
    }
    let filename = flags.config;
    if !Path::new(&filename).exists() {
        ata_message(args);
    }
    let mut contents = String::new();
    File::open(filename).unwrap().read_to_string(&mut contents).unwrap();

    let config: Config = from_str(&contents).unwrap();
    // println!("{:?}", config);

    println!("Ask the Terminal Anything. Type `commands` for a list of commands.");

    let mut rl = Editor::<()>::new()?;

    loop {
        let readline = rl.readline(&format!("{}> ", config.clone().model));
        match readline {
            Ok(line) => {
                if line == "" {
                    continue
                }
                rl.add_history_entry(line.as_str());
                let response = prompt_model(config.clone(), line)?;
                let sanitized = sanitize_response(response);
                println!("\n{}\n", sanitized);
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_newlines() {
        assert_eq!(remove_leading_newlines("\nfoo".to_string()), "foo");
    }
}
