use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use async_std::net::TcpListener;
use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use rstest::fixture;
use z2p::{
    configuration::DatabaseSettings, run, telemetry::get_subscriber, telemetry::init_subscriber,
};

pub struct App {
    pub address: SocketAddr,
    pub db: Database,
    pub db_cfg: DatabaseSettings,
    #[allow(dead_code)]
    db_container: Arc<docker::Container>,
}

fn start_db_container(
    name: &str,
    port: u16,
    timeout: Duration,
) -> docker::DockerResult<docker::Container> {
    let opts = docker::DockerOptions::default()
        .name(&name)
        .env("MONGO_INITDB_ROOT_USERNAME".to_owned(), "mongo".to_owned())
        .env(
            "MONGO_INITDB_ROOT_PASSWORD".to_owned(),
            "password".to_owned(),
        )
        .port(port, 27017);
    let mut container_result = docker::Container::run("mongo", Some(&opts));
    let end = std::time::SystemTime::now() + timeout;
    while std::time::SystemTime::now() < end
        && match container_result.as_ref() {
            Err(docker::DockerError::CannotStartInstance { stderr, .. }) => {
                stderr.contains("is already in use by container")
            }
            _ => false,
        }
    {
        std::thread::sleep(core::time::Duration::from_millis(50));
        container_result = docker::Container::run("mongo", Some(&opts));
    }
    container_result
}

const DEFAULT_DB_HOST_PORT: u16 = 37017;

#[fixture]
pub fn db_container() -> Arc<docker::Container> {
    lazy_static::lazy_static! {
        static ref DBREF: Mutex<Weak<docker::Container>> = Mutex::new(Weak::new());
    };
    let mut weak = DBREF.lock().unwrap();
    loop {
        match weak.upgrade() {
            // Some(strong) => strong.clone(),
            Some(_strong) => {}
            None => {
                eprintln!("Create new container");
                let name = format!("z2p_tests_{}", DEFAULT_DB_HOST_PORT);
                let strong = Arc::new(
                    start_db_container(
                        &name,
                        DEFAULT_DB_HOST_PORT,
                        std::time::Duration::from_secs(5),
                    )
                    .unwrap(),
                );
                eprintln!("Created new container {:?}", strong);
                *weak = Arc::downgrade(&strong);
                return strong;
            }
        }
    }
}

#[fixture]
pub fn tracing() {
    lazy_static::lazy_static! {
        static ref SUBSCRIBER: () = {
            let filter = if std::env::var("TEST_LOG").is_ok() { "debug" } else { "" };
            let subscriber = get_subscriber("test", filter);
            init_subscriber(subscriber);
        };
    }
    *SUBSCRIBER
}

#[fixture(cfg=configurations())]
pub fn app(cfg: DatabaseSettings, db_container: Arc<docker::Container>, _tracing: ()) -> App {
    let listener = async_std::task::block_on(async {
        TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Cannot bind server address")
    });

    let address = listener.local_addr().expect("Cannot get server address");
    let c = cfg.clone();
    async_std::task::block_on(create_db(&c));
    let c = cfg.clone();
    async_std::task::spawn(async { run(c).await.listen(listener).await });
    let db = async_std::task::block_on(db(&cfg));
    App {
        address,
        db,
        db_cfg: cfg,
        db_container,
    }
}

fn now() -> DateTime<Utc> {
    std::time::SystemTime::now().into()
}

fn testname() -> String {
    std::thread::current().name().unwrap().to_string()
}

fn sanitize_db_name(name: impl AsRef<str>) -> String {
    const MAX_DB_NAME_LEN: usize = 70;
    const DB_START_NAME_LEN: usize = 20;
    const DB_END_NAME_LEN: usize = 15;
    let mut name = name.as_ref().replace(
        &['/', '\\', '.', '“', '*', '<', '>', ':', '|', '?', '$'][..],
        "_",
    );
    let l = name.len();
    if l > MAX_DB_NAME_LEN {
        let start = &name[0..DB_START_NAME_LEN];
        let end = &name[(l - DB_END_NAME_LEN)..l];
        name = format!("{}={}={}", start, now().timestamp_millis() % 60000, end);
    }
    name
}

pub fn configurations() -> DatabaseSettings {
    let mut configurations =
        z2p::configuration::get_configuration().expect("Failed to read configurations");
    configurations.database.name = sanitize_db_name(testname());
    configurations.database.port = DEFAULT_DB_HOST_PORT;
    configurations.database
}

async fn mongodb_client_options(url: &str) -> ClientOptions {
    let mut client_options = ClientOptions::parse(&url)
        .await
        .expect("Cannot parse db connection string");
    client_options.server_selection_timeout = Some(Duration::from_millis(5000));
    client_options.connect_timeout = Some(Duration::from_millis(5000));

    client_options
}

async fn create_db(cfg: &DatabaseSettings) {
    let url = format!(
        "mongodb://{}:{}@{}:{}",
        cfg.username, cfg.password, cfg.host, cfg.port
    );
    let mut client_options = mongodb_client_options(&url).await;
    client_options.app_name = Some("CreateDb".to_string());

    let client = Client::with_options(client_options).expect("Cannot create db client");
    let db = client.database(&cfg.name);

    let collection = db.collection("test_entry__");

    let doc = doc! { "test_name": testname(), "created": now() };

    // Insert some documents into the "mydb.books" collection.
    collection
        .insert_one(doc, None)
        .await
        .expect("Cannot write new db");
}

async fn db(cfg: &DatabaseSettings) -> Database {
    let mut client_options = mongodb_client_options(&cfg.connection_string()).await;
    client_options.app_name = Some(testname());

    Client::with_options(client_options)
        .expect("Cannot create db client")
        .database(&cfg.name)
}

pub mod docker {

    use std::{collections::HashMap, process::Command};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum DockerError {
        #[error(
            "Cannot create image '{name}' [ code: {code:?} stdout: '{stdout}'  stdout: '{stderr}'"
        )]
        CannotCreateImage {
            name: String,
            code: Option<i32>,
            stdout: String,
            stderr: String,
        },

        #[error("Cannot start instance : image '{image}' [ code: {code:?} stdout: '{stdout}'  stdout: '{stderr}'")]
        CannotStartInstance {
            image: String,
            code: Option<i32>,
            stdout: String,
            stderr: String,
        },

        #[error("Command execution error")]
        Exec(#[from] std::io::Error),

        #[error("Command error")]
        CmdErr(std::process::Output),

        #[error("Unknow Error: '{0}'")]
        Unknow(String),
    }

    pub type DockerResult<T> = Result<T, DockerError>;

    #[derive(Debug, Clone, Default)]
    pub struct DockerOptions {
        name: Option<String>,
        envs: HashMap<String, String>,
        ports: HashMap<u16, u16>,
    }

    impl DockerOptions {
        pub fn name(mut self, name: &str) -> Self {
            let name = name.trim();
            self.name = if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            };
            self
        }

        pub fn env(mut self, key: String, value: String) -> Self {
            self.envs.insert(key, value);
            self
        }

        pub fn port(mut self, from: u16, to: u16) -> Self {
            self.ports.insert(from, to);
            self
        }

        fn add_args(&self, mut cmd: Command) -> Command {
            if let Some(name) = &self.name {
                cmd.args(&["--name", name]);
            }
            for (k, v) in &self.envs {
                cmd.args(&["-e", &format!("{}={}", k, v)]);
            }
            for (f, t) in &self.ports {
                cmd.args(&["-p", &format!("{}:{}", f, t)]);
            }
            cmd
        }
    }

    #[derive(Debug)]
    pub struct Container {
        id: String,
    }

    impl Container {
        pub fn run(image: impl AsRef<str>, options: Option<&DockerOptions>) -> DockerResult<Self> {
            let image = image.as_ref();
            let mut cmd = Self::docker_run(options);
            cmd.arg("-d").arg(image);
            cmd.output().map_err(|e| e.into()).and_then(|out| {
                if out.status.success() {
                    Ok(Self {
                        id: String::from_utf8_lossy(&out.stdout).trim().to_owned(),
                    })
                } else {
                    Err(DockerError::CannotStartInstance {
                        image: image.to_owned(),
                        code: out.status.code(),
                        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    })
                }
            })
        }

        fn docker_run(options: Option<&DockerOptions>) -> Command {
            let mut cmd = Command::new("docker");
            cmd.arg("run");
            cmd.arg("--rm");
            if let Some(opts) = options {
                cmd = opts.add_args(cmd);
            }
            cmd
        }
    }

    impl Drop for Container {
        fn drop(&mut self) {
            let _ = Command::new("docker").arg("kill").arg(&self.id).output();
            eprintln!("Destroy container {}", &self.id);
        }
    }

    pub fn running_container(name: &str) -> DockerResult<Option<Container>> {
        let out = Command::new("docker")
            .arg("ps")
            .args(&["--filter", &format!("name={}", name)])
            .arg("-q")
            .output()?;
        if out.status.code() != Some(0) {
            return Err(DockerError::CmdErr(out));
        }
        let id = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        if id.len() > 0 {
            Ok(Some(Container { id }))
        } else {
            Ok(None)
        }
    }
}
