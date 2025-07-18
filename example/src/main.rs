use raycast_rust_macros::raycast;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[raycast]
fn noop() {
    // No-op function
}

#[raycast]
fn greeting(name: String, is_formal: bool) -> String {
    format!("Hello {}{name}!", if is_formal { "Mr/Ms " } else { "" })
}

#[raycast]
fn greetings(names: Vec<String>) -> Vec<String> {
    names.into_iter().map(|name| format!("Hello {name}!")).collect()
}

#[raycast]
async fn delayed_greeting(name: String, seconds: f64) -> Result<String, String> {
    if seconds < 0.0 {
        return Err("Seconds must be non-negative".to_string());
    }

    tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
    Ok(format!("... Hello {name}!"))
}

#[raycast]
fn optionals(value: Option<String>) -> Option<String> {
    value.map(|v| format!("Got: {v}"))
}

#[raycast]
fn pick_color(name: String) -> Result<Color, String> {
    match name.as_str() {
        "red" => Ok(Color { red: 1.0, green: 0.0, blue: 0.0 }),
        "green" => Ok(Color { red: 0.0, green: 1.0, blue: 0.0 }),
        "blue" => Ok(Color { red: 0.0, green: 0.0, blue: 1.0 }),
        _ => Err(format!("{name} is not a supported color")),
    }
}

#[derive(Deserialize, Serialize)]
struct Color {
    red: f32,
    green: f32,
    blue: f32,
}
