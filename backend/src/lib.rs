use std::net::Ipv6Addr;
use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use std::sync::Arc;

use futures_util::{StreamExt, Future};
use quinn::{Endpoint, ServerConfig, NewConnection, Incoming};
use rustls::RootCertStore;
use rustls::server::{ResolvesServerCert, ResolvesServerCertUsingSni, AllowAnyAuthenticatedClient};


fn generate_self_signed_cert() -> Result<(rustls::Certificate, rustls::PrivateKey), Box<dyn Error>>
{
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = rustls::PrivateKey(cert.serialize_private_key_der());
    Ok((rustls::Certificate(cert.serialize_der()?), key))
}

use std::{error::Error, fs::File, io::BufReader};

pub fn read_certs_from_file(
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open("./certificates.pem")?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open("./privkey.pem")?);
    // if the file starts with "BEGIN RSA PRIVATE KEY"
    // let mut key_vec = rustls_pemfile::rsa_private_keys(&mut reader)?;
    // if the file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}

pub struct MyResolvesClientCert {

}

impl rustls::client::ResolvesClientCert for MyResolvesClientCert {
    fn resolve(
        &self,
        acceptable_issuers: &[&[u8]],
        sigschemes: &[rustls::SignatureScheme],
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        todo!()
    }

    fn has_certs(&self) -> bool {
        todo!()
    }
}

pub async fn setup<F, Fut>(f: F) -> Result<(), Box<dyn Error>> 
where
    F: FnOnce(Endpoint) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>> {
    let addr = "[::1]:0".parse()?;
    let (cert, key) = generate_self_signed_cert()?;

    let client_root_store = RootCertStore::empty();
    let server_root_store = RootCertStore::empty();

    // vec![cert], key)?
    
    // with_client_cert_verifier would need to be per SNI
    let server_config = rustls::server::ServerConfig::builder().with_safe_defaults().with_client_cert_verifier(AllowAnyAuthenticatedClient::new(server_root_store)).with_cert_resolver(Arc::new(ResolvesServerCertUsingSni::new()));
    let client_config = rustls::client::ClientConfig::builder().with_safe_defaults().with_root_certificates(client_root_store).with_client_cert_resolver(Arc::new(MyResolvesClientCert{}));

    let (mut endpoint, mut incoming) = Endpoint::server(ServerConfig::with_crypto(Arc::new(server_config)), addr)?;
    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_config)));

    println!("{}", endpoint.local_addr()?);

    let res = tokio::try_join!(
        f(endpoint),
        handle_incoming(incoming));

    res?;

    Ok(())
}

pub async fn handle_incoming(mut incoming: Incoming) -> Result<(), Box<dyn Error>> {
    while let Some(conn) = incoming.next().await {
        let mut connection: NewConnection = conn.await?;

        println!("connection");

        // Save connection somewhere, start transferring, receiving data, see DataTransfer tutorial.
    }
    Ok(())
}

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    CString::new("Hello ".to_owned() + recipient).unwrap().into_raw()
}

/// Expose the JNI interface for android below
#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    use super::*;
    use jni::JNIEnv;
    use jni::objects::{JClass, JString};
    use jni::sys::{jstring};

    #[no_mangle]
    pub unsafe extern fn Java_de_selfmade4u_rustapplication_RustGreetings_greeting(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
        // Our Java companion code might pass-in "world" as a string, hence the name.
        let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let world_ptr = CString::from_raw(world);
        let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

        output.into_inner()
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
