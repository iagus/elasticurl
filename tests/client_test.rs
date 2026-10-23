use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use elasticurl::client::Client;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn get_json_returns_parsed_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/_cluster/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "green"})))
        .mount(&server)
        .await;

    let client = Client::new(&server.uri(), None, false);
    let value = client.get_json("/_cluster/health", &[]).unwrap();
    assert_eq!(value, json!({"status": "green"}));
}

#[tokio::test]
async fn error_reason_is_extracted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/nope/_mapping"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": {"type": "index_not_found_exception", "reason": "no such index [nope]"},
            "status": 404
        })))
        .mount(&server)
        .await;

    let client = Client::new(&server.uri(), None, false);
    let err = client.get_json("/nope/_mapping", &[]).unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("no such index [nope]"), "got: {msg}");
}

#[tokio::test]
async fn sends_basic_auth_header() {
    let server = MockServer::start().await;
    let expected = format!("Basic {}", BASE64.encode("elastic:secret"));
    Mock::given(method("GET"))
        .and(path("/_cat/aliases"))
        .and(header("authorization", expected.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_string("alias index\n"))
        .mount(&server)
        .await;

    let client = Client::new(&server.uri(), Some("elastic:secret".to_string()), false);
    let text = client.get_text("/_cat/aliases", &[]).unwrap();
    assert!(text.contains("alias index"));
}

#[tokio::test]
async fn query_params_are_url_encoded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/logs/_search"))
        .and(query_param("q", "status code:404"))
        .and(query_param("size", "5"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"hits": {"total": {"value": 0}, "hits": []}})),
        )
        .mount(&server)
        .await;

    let client = Client::new(&server.uri(), None, false);
    let params = vec![
        ("q".to_string(), "status code:404".to_string()),
        ("size".to_string(), "5".to_string()),
    ];
    let value = client.get_json("/logs/_search", &params).unwrap();
    assert_eq!(value.pointer("/hits/total/value"), Some(&json!(0)));
}

#[tokio::test]
async fn post_sends_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/logs/_search"))
        .and(body_json(json!({"query": {"match_all": {}}, "size": 3})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"hits": {"total": {"value": 0}, "hits": []}})),
        )
        .mount(&server)
        .await;

    let client = Client::new(&server.uri(), None, false);
    let value = client
        .post_json("/logs/_search", &json!({"query": {"match_all": {}}, "size": 3}))
        .unwrap();
    assert!(value.pointer("/hits").is_some());
}
