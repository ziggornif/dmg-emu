use std::fs;

use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use emulator::gameboy::Gameboy;
use tokio::net::TcpListener;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeTemplate {}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut gameboy = Gameboy::new();
    match fs::read("debug-ui/resources/cpu_instrs.gb") {
        Ok(rom_data) => match gameboy.load_rom(&rom_data) {
            Ok(_) => println!("ROM loaded successfully!"),
            Err(e) => panic!("Error: {}", e),
        },
        Err(_) => {
            panic!("ROM not found");
        }
    }

    let mut opcodes = format!("0x{:02X}\n", gameboy.bus.read_byte(0));
    for i in 1..10000 {
        opcodes += &format!("0x{:02X}\n", gameboy.bus.read_byte(i));
    }
    match fs::write("opcodes.txt", opcodes) {
        Ok(_) => println!("Opcodes writed in opcodes.txt"),
        Err(e) => println!("Could not write opcodes file {}", e),
    }
    // println!("0x{:02X}", gameboy.bus.read_byte(gameboy.cpu.pc + 1));
    // println!("0x{:02X}", gameboy.bus.read_byte(gameboy.cpu.pc + 2));
    // println!("0x{:02X}", gameboy.bus.read_byte(gameboy.cpu.pc + 3));
    // println!("0x{:04X}", gameboy.bus.read_byte(0x0637));
    let app = Router::new().route("/", get(index)).with_state(gameboy);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> impl IntoResponse {
    HtmlTemplate(HomeTemplate {})
}
