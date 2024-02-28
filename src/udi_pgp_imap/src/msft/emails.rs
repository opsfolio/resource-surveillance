use graph_rs_sdk::{oauth::AccessToken, Graph, ODataQuery};

pub async fn fetch_emails_from_graph_api(token: &AccessToken) -> anyhow::Result<()> {
    let client = Graph::new(token.bearer_token());
    let folders = client
        .me()
        .mail_folders()
        .list_mail_folders()
        .send()
        .await?;
    let body: serde_json::Value = folders.json().await?;

    // let res = client
    //     .me()
    //     .mail_folder("jj")
    //     .messages()
    //     .list_messages()
    //     .top("1000").paging()
    //     .send()
    //     .await?;

    Ok(())
}
