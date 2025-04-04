use anyhow::anyhow;

//use crate::hyperware::process::sign;
use hyperware_process_lib::logging::{init_logging, Level};
use hyperware_process_lib::net::{NetAction, NetResponse};
use hyperware_process_lib::{
    our, Address, LazyLoadBlob, Request,
};

//use hyperware_app_common::send;
use hyperprocess_macro::hyperprocess;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct SignState {}

async fn sign(message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let message = make_message(source, &message);

    let res = Request::to(("our", "net", "distro", "sys"))
        .blob(LazyLoadBlob {
            mime: None,
            bytes: message,
        })
        .body(body)
        .send_and_await_response(10)??;

    let Ok(NetResponse::Signed) = rmp_serde::from_slice::<NetResponse>(res.body()) else {
        return Err(anyhow!("signature failed"));
    };
    let Some(signature) = res.blob() else {
        return Err(anyhow!("no blob"));
    };

    Ok(signature)
}

async fn verify(message: Vec<u8>, signature: Vec<u8>) -> anyhow::Result<bool> {
    let message = make_message(source, &message);
    let body = rmp_serde::to_vec(&NetAction::Verify {
        from: our(),
        signature,
    })?;

    let res = Request::to(("our", "net", "distro", "sys"))
        .blob(LazyLoadBlob {
            mime: None,
            bytes: message,
        })
        .body(body)
        .send_and_await_response(10)??;

    let resp = rmp_serde::from_slice::<NetResponse>(res.body())?;

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
fn make_message(source: &Address, bytes: &Vec<u8>) -> Vec<u8> {
    [source.to_string().as_bytes(), &bytes].concat()
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
            Err(e) => Err(e.to_string),
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
