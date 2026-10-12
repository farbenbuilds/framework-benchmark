use actix_web::{HttpResponse, Result, error::ErrorInternalServerError, web};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AppState {
    pub pool: Pool,
    pub sequence: AtomicU64,
}

#[derive(Deserialize)]
pub struct QueryCount {
    #[serde(default = "default_query_count")]
    pub queries: u16,
}

#[derive(Serialize)]
struct World {
    id: i32,
    randomnumber: i32,
}

struct Fortune {
    id: i32,
    message: String,
}

pub async fn db_single(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut client = state.pool.get().await.map_err(ErrorInternalServerError)?;
    let world = fetch_world(&mut client, next_id(&state)).await?;
    Ok(HttpResponse::Ok().json(world))
}

pub async fn db_multi(
    state: web::Data<AppState>,
    count: web::Query<QueryCount>,
) -> Result<HttpResponse> {
    let mut client = state.pool.get().await.map_err(ErrorInternalServerError)?;
    let count = normalized_count(count.queries);
    let mut worlds = Vec::with_capacity(count);
    for _ in 0..count {
        worlds.push(fetch_world(&mut client, next_id(&state)).await?);
    }
    Ok(HttpResponse::Ok().json(worlds))
}

pub async fn fortunes(state: web::Data<AppState>) -> Result<HttpResponse> {
    let client = state.pool.get().await.map_err(ErrorInternalServerError)?;
    let rows = client
        .query("SELECT id, message FROM fortune", &[])
        .await
        .map_err(ErrorInternalServerError)?;
    let mut fortunes: Vec<_> = rows
        .iter()
        .map(|row| Fortune {
            id: row.get(0),
            message: row.get(1),
        })
        .collect();
    fortunes.push(Fortune {
        id: 0,
        message: "Additional fortune added at request time.".into(),
    });
    fortunes.sort_unstable_by(|left, right| left.message.cmp(&right.message));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_fortunes(&fortunes)))
}

pub async fn updates(
    state: web::Data<AppState>,
    count: web::Query<QueryCount>,
) -> Result<HttpResponse> {
    let mut client = state.pool.get().await.map_err(ErrorInternalServerError)?;
    let transaction = client
        .transaction()
        .await
        .map_err(ErrorInternalServerError)?;
    let count = normalized_count(count.queries);
    let mut worlds = Vec::with_capacity(count);
    let mut ids = Vec::with_capacity(count);
    let mut randomnumbers = Vec::with_capacity(count);
    for _ in 0..count {
        let id = next_id(&state);
        let row = transaction
            .query_one("SELECT id, randomnumber FROM world WHERE id = $1", &[&id])
            .await
            .map_err(ErrorInternalServerError)?;
        let randomnumber = next_id(&state);
        ids.push(id);
        randomnumbers.push(randomnumber);
        worlds.push(World {
            id: row.get(0),
            randomnumber,
        });
    }
    transaction
        .execute(
            "UPDATE world SET randomnumber = changes.randomnumber \
             FROM unnest($1::int[], $2::int[]) AS changes(id, randomnumber) \
             WHERE world.id = changes.id",
            &[&ids, &randomnumbers],
        )
        .await
        .map_err(ErrorInternalServerError)?;
    transaction
        .commit()
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(worlds))
}

async fn fetch_world(client: &mut deadpool_postgres::Client, id: i32) -> Result<World> {
    let row = client
        .query_one("SELECT id, randomnumber FROM world WHERE id = $1", &[&id])
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(World {
        id: row.get(0),
        randomnumber: row.get(1),
    })
}

fn normalized_count(count: u16) -> usize {
    usize::from(count.clamp(1, 500))
}

fn default_query_count() -> u16 {
    1
}

fn next_id(state: &AppState) -> i32 {
    let mut current = state.sequence.load(Ordering::Relaxed);
    loop {
        let next = current ^ (current << 13) ^ (current >> 7) ^ (current << 17);
        match state.sequence.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return (next % 10_000 + 1) as i32,
            Err(value) => current = value,
        }
    }
}

fn render_fortunes(fortunes: &[Fortune]) -> String {
    let mut html =
        String::from("<!DOCTYPE html><html><body><table><tr><th>id</th><th>message</th></tr>");
    for fortune in fortunes {
        html.push_str("<tr><td>");
        html.push_str(&fortune.id.to_string());
        html.push_str("</td><td>");
        escape_html(&fortune.message, &mut html);
        html.push_str("</td></tr>");
    }
    html.push_str("</table></body></html>");
    html
}

fn escape_html(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_html, normalized_count};

    #[test]
    fn normalizes_query_count() {
        assert_eq!(normalized_count(0), 1);
        assert_eq!(normalized_count(20), 20);
        assert_eq!(normalized_count(501), 500);
    }

    #[test]
    fn escapes_fortune_html() {
        let mut output = String::new();
        escape_html("<&>\"'", &mut output);
        assert_eq!(output, "&lt;&amp;&gt;&quot;&#x27;");
    }
}
