use axum::{response::Html, routing::get, Json, Router};
use serde_json::{json, Value};

const PUERTO: u16 = 8080;

// El mock vive en docs/, fuera del contexto de build (que es app/), asi que
// deploy.yml lo copia aqui antes de construir la imagen. Si no esta, se sirve
// el texto plano de siempre: el backend nunca depende de que exista.
const MOCK: &str = "/app/mock.html";
const RESPALDO: &str = "PruebaInit backend";

async fn salud() -> Json<Value> {
    let build = std::env::var("BUILD_ID").unwrap_or_else(|_| "dev".to_string());
    Json(json!({ "ok": true, "build": build }))
}

#[tokio::main]
async fn main() {
    // Se lee una sola vez al arrancar, no en cada peticion. MOCK_HTML permite
    // apuntar a otra ruta para probarlo fuera del contenedor.
    let ruta = std::env::var("MOCK_HTML").unwrap_or_else(|_| MOCK.to_string());
    let mock: &'static str = match std::fs::read_to_string(&ruta) {
        Ok(html) => Box::leak(html.into_boxed_str()),
        Err(e) => {
            println!("sin {ruta} ({e}): se sirve texto plano");
            RESPALDO
        }
    };

    let app = Router::new()
        .route("/", get(move || async move { Html(mock) }))
        .route("/salud", get(salud));

    let direccion = format!("0.0.0.0:{PUERTO}");
    let listener = tokio::net::TcpListener::bind(&direccion)
        .await
        .unwrap_or_else(|e| panic!("no se pudo abrir {direccion}: {e}"));

    println!("PruebaInit backend escuchando en {direccion}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("fallo del servidor HTTP");
}
