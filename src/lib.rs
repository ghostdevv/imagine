use worker::*;

mod gif;

#[event(fetch)]
async fn fetch(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let imagine = _env.bucket("IMAGINE").unwrap();

    let list = imagine.list().execute().await?;
    console_log!("objects: {}", list.objects().len());

    // Get the SpongeBob GIF from the bucket
    let spongebob_gif_data = imagine
        .get("base.gif")
        .execute()
        .await
        .unwrap()
        .unwrap()
        .body()
        .unwrap()
        .bytes()
        .await
        .unwrap();

    // Add text to the GIF
    let text = "IMAGINE";
    let modified_gif = gif::add_text_to_gif(&spongebob_gif_data, text)?;

    // Return the modified GIF
    Response::from_bytes(modified_gif).map(|mut res| {
        res.headers_mut().set("Content-Type", "image/gif").unwrap();
        res
    })
}
