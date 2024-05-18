use flume::unbounded;
use serde::Deserialize;

const WORKERS: usize = 4;

struct Repo(String);
struct RepoWithDesc(String, String);

async fn fetch_desc(Repo(repo): Repo) -> RepoWithDesc {
    #[derive(Deserialize)]
    pub struct Json {
        full_name: String,
        description: String,
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0")
        .build()
        .unwrap();

    let body = client
        .get(format!("https://api.github.com/repos/{repo}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let Json {
        full_name,
        description,
    } = serde_json::from_str(&body).unwrap();
    RepoWithDesc(full_name, description)
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = unbounded(); // task
    let (result, collector) = unbounded(); // result

    for _ in 0..WORKERS {
        let receiver = receiver.clone();
        let result = result.clone();
        tokio::task::spawn(async move {
            while let Ok(repo) = receiver.recv_async().await {
                result.send(fetch_desc(repo).await).unwrap();
            }
        });
    }
    std::mem::drop(receiver);
    std::mem::drop(result);

    let tasks = vec![
        "SeaQL/sea-orm",
        "SeaQL/sea-query",
        "SeaQL/sea-schema",
        "SeaQL/sea-streamer",
        "SeaQL/seaography",
        "SeaQL/starfish-ql",
        "SeaQL/FireDBG.for.Rust",
        "SeaQL/sea-orm-tutorial",
        "SeaQL/sea-orm-cookbook",
    ];

    for task in &tasks {
        sender.send(Repo(task.to_string())).unwrap();
    }

    for _ in 0..tasks.len() {
        let RepoWithDesc(repo, desc) = collector.recv_async().await.unwrap();
        println!("{repo}: {desc}");
    }
}
