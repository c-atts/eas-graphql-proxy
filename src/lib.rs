use std::collections::HashMap;

use lazy_static::lazy_static;
use worker::*;

lazy_static! {
    static ref EAS_CHAIN_GQL_ENDPOINT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("optimism", "https://optimism.easscan.org/graphql");
        m.insert("sepolia", "https://sepolia.easscan.org/graphql");
        m.insert("base", "https://base.easscan.org/graphql");
        m
    };
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    let router = Router::new();
    router
        .post_async("/graphql/:chain", |mut req, ctx| async move {
            if let Some(chain) = ctx.param("chain") {
                if let Some(uri) = EAS_CHAIN_GQL_ENDPOINT.get(chain.as_str()) {
                    let body = req.text().await?;

                    let mut headers = Headers::new();
                    headers.append("Content-Type", "application/json")?;
                    headers.append("User-Agent", "c-atts/0.0.1")?;

                    let mut init = RequestInit::new();
                    init.with_headers(headers);
                    init.with_method(Method::Post);
                    init.with_body(Some(body.into()));

                    let request = Request::new_with_init(uri, &init)?;
                    return Fetch::Request(request).send().await;
                }
                return Response::error("Chain not supported", 400);
            }
            Response::error("Chain parameter missing", 404)
        })
        .run(req, env)
        .await
}
