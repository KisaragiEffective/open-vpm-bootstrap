use std::io::{BufWriter, Read, Write};
use std::ops::Deref;
use url::Url;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

fn main() {
    let client = reqwest::blocking::ClientBuilder::new();
    let client = client
        .gzip(true)
        .user_agent("reqwest/1.0 open-vpm-bootstrap/1.0 https://github.com/KisaragiEffective/open-vpm-bootstrap")
        .build().expect("failed to initialize HTTP client");

    let req = client.get("https://api.vrchat.cloud/api/1/config").build().expect("failed to construct HTTP request");

    let bootstrap_info = client.execute(req)
        .expect("failed to send HTTP request")
        .json::<VRChatApiEndpointResponse<VRChatPackageManagerConfig>>()
        .expect("failed to parse JSON")
        .expect("fatal");

    let bootstrap_package_url = bootstrap_info.bootstrap_install.url;

    let req = client.get(bootstrap_package_url).build().expect("failed to construct HTTP request");

    let unity_package = client.execute(req)
        .expect("failed to send HTTP request")
        .bytes()
        .expect("failed to load response");

    let out_file = NamedTempFileDropper(NamedTempFile::new().expect("failed to create temporary directory"));
    let mut bw = BufWriter::new(out_file.deref());

    bw.write_all(&unity_package).expect("failed to write");

    println!("Please import {} to your editor.", out_file.path().display());
    println!("Press enter to quit.");

    std::io::stdin().read(&mut []).unwrap_or_default();
}

#[derive(Deserialize)]
struct VRChatPackageManagerConfig {
    // This is not full list
    #[serde(rename = "downloadUrls")]
    bootstrap_install: BootstrapBinaryDistribution
}

#[derive(Deserialize)]
struct BootstrapBinaryDistribution {
    #[serde(rename = "bootstrap")]
    url: Url,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum VRChatApiEndpointResponse<T> {
    Err {
        error: VRChatApiEndpointError,
    },
    Ok(T),
}

impl<T> VRChatApiEndpointResponse<T> {
    fn expect(self, message: &str) -> T {
        match self {
            VRChatApiEndpointResponse::Err { error } => {
                panic!("VRChat endpoint error: {}", error.message())
            }
            VRChatApiEndpointResponse::Ok(t) => t,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum VRChatApiEndpointError {
    RejectedByWaf {
        message: String,
        // identify error = 13799
        waf_code: u32,
    },
    Plain {
        message: String,
    },
}

impl VRChatApiEndpointError {
    fn message(&self) -> &str {
        match self {
            VRChatApiEndpointError::RejectedByWaf { message, .. } => message,
            VRChatApiEndpointError::Plain { message } => message,
        }
    }
}

struct NamedTempFileDropper(NamedTempFile);

impl Drop for NamedTempFileDropper {
    fn drop(&mut self) {
        if let Err(x) = std::fs::remove_file(self.0.path()) {
            eprintln!("failed to remove temporary file: {x}")
        }
    }
}

impl Deref for NamedTempFileDropper {
    type Target = NamedTempFile;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
