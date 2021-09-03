use std::fs::File;
use std::io::Read;
use std::sync::Mutex;

use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, web};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::expander::Expander;
use crate::global_alias::GlobalAliasTable;
use crate::meta::Meta;


struct AppData {
    db: Database,
    aliases: GlobalAliasTable,
}

#[derive(Deserialize)]
struct FileQuery {
    path: String
}

#[derive(Deserialize)]
struct SearchQuery {
    expression: String
}

#[derive(Serialize)]
struct QueryResult {
    items: Vec<Meta>,
    expression: String,
}

async fn search(data: web::Data<Mutex<AppData>>, query: web::Json<SearchQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let expression = expander.expand(&query.expression);

    let mut items: Vec<Meta> = vec![];

    data.db.select(&expression, false, |meta, _vacuumed| {
        items.push(meta.clone());
        Ok(())
    })?;

    Ok(HttpResponse::Ok().json(QueryResult { items, expression }))
}

async fn file(data: web::Data<Mutex<AppData>>, query: web::Query<FileQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock file");
    let found = data.db.get(&query.path)?;
    let found = found.ok_or(AppError::Void)?;
    let mut content: Vec<u8> = vec![];
    let mut file = File::open(&found.file.path)?;
    file.read_to_end(&mut content)?;
    let content_type = format!("image/{}", found.format);
    Ok(HttpResponse::Ok().content_type(content_type).body(content))
}

#[actix_web::main]
pub async fn start(db: Database, aliases: GlobalAliasTable, port: u16) -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppData{aliases, db}));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::resource("/search").route(web::post().to(search)))
            .service(web::resource("/file").route(web::get().to(file)))
            .service(Files::new("/", "static").index_file("index.html"))
    }).bind(("0.0.0.0", port))?.run().await
}
