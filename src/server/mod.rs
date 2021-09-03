use std::fs::File;
use std::io::Read;
use std::sync::Mutex;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};

use crate::database::Database;
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

async fn index(data: web::Data<Mutex<AppData>>, query: web::Json<SearchQuery>) -> impl Responder {
    let data = data.lock().unwrap();
    let expander = Expander::generate(&data.db, &data.aliases).unwrap(); // FIXME
    let expression = expander.expand(&query.expression);

    let mut items: Vec<Meta> = vec![];

    data.db.select(&expression, false, |meta, _vacuumed| {
        items.push(meta.clone());
        Ok(())
    }).expect("select"); // FIXME

    HttpResponse::Ok().json(QueryResult { items, expression })
}

async fn file(data: web::Data<Mutex<AppData>>, query: web::Query<FileQuery>) -> impl Responder {
    let data = data.lock().unwrap();
    if let Ok(found) = data.db.get(&query.path) {
        if let Some(found) = found {
            let mut content: Vec<u8> = vec![];
            let mut file = File::open(&found.file.path).expect("Could not open");
            file.read_to_end(&mut content).expect("Could not read");
            let content_type = format!("image/{}", found.format);
            HttpResponse::Ok().content_type(content_type).body(content)
        } else {
            HttpResponse::NotFound().body("File not found")
        }
    } else {
        HttpResponse::BadRequest().body("Bad request")
    }
}

#[actix_web::main]
pub async fn start(db: Database, aliases: GlobalAliasTable) -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppData{aliases, db}));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::resource("/").route(web::post().to(index)))
            .service(web::resource("/file").route(web::get().to(file)))
    }).bind(("0.0.0.0", 8080))?.run().await
}
