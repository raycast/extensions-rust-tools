# Rust for Raycast Extensions

This Rust Crate contains code generation macros and plugins to build a communication channel between [Raycast](https://raycast.com)'s React extensions and Rust native code. Basically, it lets you import Rust code into your Raycast extension in order to:

- leverage native Windows APIs that might not be exposed to TS/JS, or
- compartmentalize your extension into client-facing code (react) and system code (Rust).

### Requirements

- [Rustup](https://rustup.rs/)
- Install Rust Windows msvc target via rustup: `rustup target add x86_64-pc-windows-msvc`

## Using the Package

We built a sample extension using Rust [here](https://github.com/raycast/extensions-rust-sample). Check it out to quickly get a feeling how things should be laid out.

To use Rust within Raycast:

1. Create (or fork) a Raycast extension.

   If you don't know how, check out [this guide](https://developers.raycast.com/basics/create-your-first-extension).

2. Create a Rust crate in the folder of your Raycast extension.

   ```bash
   mkdir -p rust/src
   touch rust/Cargo.toml
   touch rust/src/main.rs
   ```

   The crate should have a bin section:

   ```toml
   [package]
   name = "my-rust-api"
   version = "0.1.0"
   edition = "2021"

   [[bin]]
   name = "my-rust-api"
   path = "src/main.rs"
   ```

3. Modify the `Cargo.toml` file to include the necessary macros.

   ```diff
   + [dependencies]
   + raycast-rust-macros = { git = "https://github.com/raycast/extensions-rust-tools", package = "raycast-rust-macros", branch = "main" }
   + raycast-rust-runtime = { git = "https://github.com/raycast/extensions-rust-tools", package = "raycast-rust-runtime", branch = "main" }
   + serde = { version = "1.0", features = ["derive"] }
   + serde_json = "1.0"
   + tokio = { version = "1.0", features = ["full"] }
   ```

4. Import `raycast_rust_macros` in your Rust file.

   ```rust
   use raycast_rust_macros::raycast;
   ```

5. Write global Rust functions and mark them with the `#[raycast]` macro.

   Global functions marked with `#[raycast]` are exported to TypeScript. These functions can have any number of parameters, and one or no return type. Exported functions can also be asynchronous (`async`) or throw errors (when returning a `Result`).

   ```swift
   #[raycast]
   fn greeting(name: String) -> String {
       format!("Hello {name}!")
   }
   ```

   Custom types can be received as parameters or returned by the function. You just need to be sure the type conforms to `Deserialize` (for arguments) and `Serialize` (for return types).

   ```rust
   use serde::{Deserialize, Serialize};

   #[raycast]
   fn pick_color(name: String) -> Result<Color, String> {
       match name.as_str() {
           "red" => Ok(Color { red: 1.0, green: 0.0, blue: 0.0 }),
           "green" => Ok(Color { red: 0.0, green: 1.0, blue: 0.0 }),
           "blue" => Ok(Color { red: 0.0, green: 0.0, blue: 1.0 }),
           _ => Err(format!("{name} is not a supported color")),
       }
   }

   #[derive(Serialize)]
   struct Color {
       red: f32,
       green: f32,
       blue: f32,
   }
   ```

6. Import the rust functions in your extension

```typescript
import { pick_color } from "rust:../rust";

export default function Command() {
  console.log(pick_color("red"));
}
```
