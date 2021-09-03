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
struct Query {
    expression: String
}

#[derive(Serialize)]
struct QueryResult {
    items: Vec<Meta>,
    expression: String,
}

async fn index(data: web::Data<Mutex<AppData>>, query: web::Json<Query>) -> impl Responder {
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

#[actix_web::main]
pub async fn start(db: Database, aliases: GlobalAliasTable) -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppData{aliases, db}));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::resource("/").route(
                    web::post().to(index)))
    }).bind(("0.0.0.0", 8080))?.run().await
}
