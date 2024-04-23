use lazy_static::lazy_static;
use std::collections::HashMap;
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
        .get_async("/graphql/:chain", |req, ctx| async move {
            handle_graphql_request(req, ctx).await
        })
        .post_async("/graphql/:chain", |req, ctx| async move {
            handle_graphql_request(req, ctx).await
        })
        .run(req, env)
        .await
}

pub async fn handle_graphql_request(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    if let Some(chain) = ctx.param("chain") {
        if let Some(uri) = EAS_CHAIN_GQL_ENDPOINT.get(chain.as_str()) {
            let cache_key = req.headers().get("X-Cache-Key")?.ok_or_else(|| {
                worker::Error::RustError("X-Cache-Key header is missing".to_string())
            })?;

            let body = req.text().await?;

            let mut headers = Headers::new();
            headers.append("Content-Type", "application/json")?;
            headers.append("User-Agent", "c-atts/0.0.1")?;

            let mut props = CfProperties::new();
            props.cache_key = Some(cache_key); // Not working (Enterprise only)
            props.cache_everything = Some(true);
            props.cache_ttl = Some(60);

            let mut init = RequestInit::new();
            init.with_headers(headers);
            init.with_method(Method::Post);
            init.with_body(Some(body.into()));
            init.with_cf_properties(props);

            let request = Request::new_with_init(uri, &init)?;
            return Fetch::Request(request).send().await;
        }
        return Response::error("Chain not supported", 400);
    }
    Response::error("Chain parameter missing", 404)
}
