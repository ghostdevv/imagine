use crate::parse_query::parse_gif_path;
use percent_encoding::percent_decode_str;
use worker::*;

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

async fn handle_stats_route(imagine_bucket: Bucket) -> Result<Response> {
    let mut cursor: Option<String> = None;
    let mut count = 0;

    loop {
        let mut list_request = imagine_bucket.list();

        if let Some(ref c) = cursor {
            list_request = list_request.cursor(c.clone());
        }

        let response = list_request.execute().await?;
        count += response.objects().len();

        cursor = response.cursor();
        if cursor.is_none() {
            break;
        }
    }

    Response::from_json(&serde_json::json!({ "count": count }))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.method() != Method::Get {
        return ResponseBuilder::new()
            .with_status(405)
            .ok("Method Not Allowed\n");
    }

    let url = req.url()?;

    if url.path() == "/stats" {
        let imagine_bucket = env.bucket("IMAGINE")?;
        return handle_stats_route(imagine_bucket).await;
    }

    let path = match percent_decode_str(url.path()).decode_utf8() {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => {
            return ResponseBuilder::new()
                .with_status(400)
                .ok("Invalid UTF-8 in path\n")
        }
    };

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

    let gif_config = parse_gif_path(&path);

    if gif_config.text.is_empty() {
        return Response::redirect(Url::parse(&format!("{}/imagine.gif", origin))?);
    }

    if !path.eq_ignore_ascii_case(&format!("/{}.gif", gif_config.file_name)) {
        return Response::redirect(Url::parse(&format!(
            "{}/{}.gif",
            origin, gif_config.file_name
        ))?);
    }

    let imagine_bucket = env.bucket("IMAGINE")?;

    if let Some(file) = imagine_bucket
        .get(gif_config.bucket_path.clone())
        .execute()
        .await?
    {
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

    let gif = gif::add_text_to_gif(&base_gif, &gif_config.text)?;

    imagine_bucket
        .put(gif_config.bucket_path, gif.clone())
        .execute()
        .await?;

    return gif_response(gif);
}
