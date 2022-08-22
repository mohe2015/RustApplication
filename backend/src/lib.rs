use std::net::Ipv6Addr;
use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use std::sync::Arc;

use futures_util::{StreamExt, Future};
use quinn::{Endpoint, ServerConfig, NewConnection, Incoming};
use rustls::RootCertStore;
use rustls::server::{ResolvesServerCert, ResolvesServerCertUsingSni, AllowAnyAuthenticatedClient};
use rustls::sign::CertifiedKey;


use std::{error::Error, fs::File, io::BufReader};


/*
openssl genpkey -algorithm ed25519 -out server-key.pem
openssl req -addext basicConstraints=critical,CA:FALSE -nodes -x509 -key server-key.pem -out server-cert.pem -sha256 -batch -days 3650 -subj "/CN=localhost"

openssl genpkey -algorithm ed25519 -out client-key.pem
openssl req -addext basicConstraints=critical,CA:FALSE -nodes -x509 -key client-key.pem -out client-cert.pem -sha256 -batch -days 3650 -subj "/CN=localhost"

*/
pub fn read_certs_from_file(basename: &str
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open(format!("{basename}-cert.pem"))?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open(format!("{basename}-key.pem"))?);
  
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}

pub struct MyResolvesClientCert(Arc<rustls::sign::CertifiedKey>);

impl rustls::client::ResolvesClientCert for MyResolvesClientCert {
    fn resolve(
        &self,
        acceptable_issuers: &[&[u8]],
        sigschemes: &[rustls::SignatureScheme],
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        Some(self.0.clone())
    }

    fn has_certs(&self) -> bool {
        true
    }
}

pub async fn setup<F, Fut>(basename: &str, f: F) -> Result<(), Box<dyn Error>> 
where
    F: FnOnce(Endpoint) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>> {

    let addr = if basename == "client" { "[::1]:59881" } else { "[::1]:59882" }.parse()?;
    //let addr = "[::1]:0".parse()?;


    let (cert, key) = read_certs_from_file(basename)?;

    let (node1_cert, _) = read_certs_from_file("client")?;
    let (node2_cert, _) = read_certs_from_file("server")?;

    let mut client_root_store = RootCertStore::empty();
    client_root_store.add(&node1_cert[0])?;
    client_root_store.add(&node2_cert[0])?;

    let mut server_root_store = RootCertStore::empty();
    server_root_store.add(&node1_cert[0])?;
    server_root_store.add(&node2_cert[0])?;
    
    let server_key = CertifiedKey::new(cert.clone(), rustls::sign::any_eddsa_type(&key).unwrap());

    let server_config = rustls::server::ServerConfig::builder().with_safe_defaults().with_client_cert_verifier(AllowAnyAuthenticatedClient::new(server_root_store)).with_single_cert(cert, key)?;   

    let client_config = rustls::client::ClientConfig::builder().with_safe_defaults().with_root_certificates(client_root_store).with_client_cert_resolver(Arc::new(MyResolvesClientCert(Arc::new(server_key))));

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
