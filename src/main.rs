use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;
use std::process;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
struct RequestBody {
    time: u128,
    time_end: u128,
    is_region: bool,
    tags: Vec<String>,
    text: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("error: not enough arguments");
        println!("usage: grafana-notifier <env> <app_name> <version>");
        process::exit(1);
    }

    let env = &args[1];
    let app_name = &args[2];
    let version = format!("v{}", &args[3]);

    let resp = make_request(env, app_name, version);
    println!("{}", resp);
}

fn make_request(environ: &str, app_name: &str, version: String) -> String {
    let tags = vec![
        format!("env:{}", environ),
        "release".to_string(),
        app_name.to_string(),
    ];

    let now = get_current_timestamp();
    let (key, url) = get_env_config();

    let body = RequestBody {
        time: now,
        time_end: now,
        is_region: true,
        tags,
        text: format!("Deployed Service: {}\nVersion: {}", app_name, version),
    };

    let client = Client::new();
    let result = client.post(&*url).bearer_auth(&key).json(&body).send();

    return match result {
        Ok(resp) => match resp.status() {
            StatusCode::OK => format!("successfully notified release of:{}\n", app_name),
            s => format!("request was not successful\ncode: {:?}", s),
        },
        Err(err) => format!("error!: {:?}", err),
    };
}

fn get_current_timestamp() -> u128 {
    return match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => {
            println!("error: time before epoch");
            process::exit(1);
        }
    };
}

fn get_env_config() -> (String, String) {
    let key: String = match env::var("GRAFANA_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            println!("error: GRAFANA_TOKEN not set");
            process::exit(1);
        }
    };

    let url: String = match env::var("GRAFANA_ADDR") {
        Ok(base_url) => format!("{}/api/annotations", base_url),
        Err(_) => {
            println!("error: GRAFANA_ADDR not set");
            process::exit(1);
        }
    };

    return (key, url);
}
