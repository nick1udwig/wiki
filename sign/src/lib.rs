use anyhow::anyhow;

//use crate::hyperware::process::sign;
use hyperware_app_common::hyperware_process_lib as hyperware_process_lib;
use hyperware_process_lib::logging::{init_logging, Level};
use hyperware_process_lib::net::{NetAction, NetResponse};
use hyperware_process_lib::{last_blob, our, LazyLoadBlob, Request};

use hyperware_app_common::{send_rmp, SendResult};
use hyperprocess_macro::hyperprocess;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct SignState {}

async fn sign(message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let message = make_message(&message);
    let body = rmp_serde::to_vec(&NetAction::Sign)?;

    let req = Request::to(("our", "net", "distro", "sys"))
        .expects_response(5)
        .blob(LazyLoadBlob {
            mime: None,
            bytes: message,
        })
        .body(body);

    // TODO
    let _resp: NetResponse = match send_rmp(req).await {
        SendResult::Success(r) => r,
        SendResult::Timeout => return Err(anyhow!("timeout")),
        SendResult::Offline => return Err(anyhow!("offline")),
        SendResult::DeserializationError(e) => return Err(anyhow!(e)),
        SendResult::BuildError(e) => return Err(anyhow!("{e}")),
    };

    let Some(signature) = last_blob() else {
        return Err(anyhow!("no blob"));
    };

    Ok(signature.bytes)
}

async fn verify(message: Vec<u8>, signature: Vec<u8>) -> anyhow::Result<bool> {
    let message = make_message(&message);
    let body = rmp_serde::to_vec(&NetAction::Verify {
        from: our(),
        signature,
    })?;

    let req = Request::to(("our", "net", "distro", "sys"))
        .expects_response(5)
        .blob(LazyLoadBlob {
            mime: None,
            bytes: message,
        })
        .body(body);

    // TODO
    let resp: NetResponse = match send_rmp(req).await {
        SendResult::Success(r) => r,
        SendResult::Timeout => return Err(anyhow!("timeout")),
        SendResult::Offline => return Err(anyhow!("offline")),
        SendResult::DeserializationError(e) => return Err(anyhow!(e)),
        SendResult::BuildError(e) => return Err(anyhow!("{e}")),
    };

    match resp {
        NetResponse::Verified(is_good) => {
            Ok(is_good)
        }
        _ => Err(anyhow!("weird response")),
    }
}

/// net:distro:sys prepends the message to sign with the sender of the request
///
/// since any sign requests passed through sign:sign:sys will look to net:distro:sys
///  like they come from sign:sign:sys, we additionally prepend the message with
///  source here
///
/// so final message to be signed looks like
///
/// [sign-address, source, bytes].concat()
fn make_message(bytes: &Vec<u8>) -> Vec<u8> {
    [source().to_string().as_bytes(), &bytes].concat()
}

#[hyperprocess(
    name = "sign",
    ui = None,
    endpoints = vec![],
    save_config = SaveOptions::Never,
    wit_world = "sign-sys-v0",
)]
impl SignState {
    #[init]
    async fn init(&mut self) {
        init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    }

    #[local]
    async fn sign(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
        match sign(message).await {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }

    #[local]
    async fn verify(&mut self, message: Vec<u8>, signature: Vec<u8>) -> Result<bool, String> {
        match verify(message, signature).await {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
}
