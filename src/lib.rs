use percent_encoding::percent_decode_str;
use worker::*;

use crate::parse_query::parse_query;

mod gif;
mod parse_query;

fn gif_response(gif: Vec<u8>) -> Result<Response> {
    ResponseBuilder::new()
        .with_header("Cache-Control", "public, max-age=604800, s-maxage=604800")?
        .with_header("Access-Control-Allow-Origin", "*")?
        .with_header("Content-Type", "image/gif")?
        .from_bytes(gif)
        .map(|mut res| {
            res.headers_mut().set("Content-Type", "image/gif").unwrap();
            res
        })
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let url = req.url()?;

    let path = percent_decode_str(url.path())
        .decode_utf8_lossy()
        .into_owned();

    let origin = match url.port() {
        Some(port) => format!(
            "{}://{}:{}",
            url.scheme(),
            url.host_str().unwrap_or(""),
            port
        ),
        None => format!("{}://{}", url.scheme(), url.host_str().unwrap_or("")),
    };

    if path == "/" {
        return Response::redirect(Url::parse(&format!("{}/imagine.gif", origin))?);
    }

    if !path.trim().ends_with(".gif") {
        return ResponseBuilder::new().with_status(404).ok("not found\n");
    }

    let query = parse_query(&path);

    if query.name.is_empty() {
        return Response::redirect(Url::parse(&format!("{}/imagine.gif", origin))?);
    }

    // Check if URL needs normalization
    let path_key = if path.len() > 5 {
        &path[1..path.len() - 4]
    } else {
        ""
    };

    if path_key != query.key {
        let normalized_url = format!("{}/{}.gif", origin, query.key);
        return Response::redirect(Url::parse(&normalized_url)?);
    }

    let imagine_bucket = env.bucket("IMAGINE")?;

    if let Some(file) = imagine_bucket.get(query.key_path.clone()).execute().await? {
        let gif = file.body().unwrap().bytes().await?;

        return gif_response(gif);
    }

    let base_gif = imagine_bucket
        .get("base.gif")
        .execute()
        .await?
        .unwrap()
        .body()
        .unwrap()
        .bytes()
        .await?;

    let gif = gif::add_text_to_gif(&base_gif, &query.name)?;

    imagine_bucket
        .put(query.key_path, gif.clone())
        .execute()
        .await?;

    return gif_response(gif);
}
