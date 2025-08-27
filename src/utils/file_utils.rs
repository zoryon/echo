use actix_web::HttpResponse;
use ssh2::Session;
use std::io::{Read, Write};
use actix_web::web::Bytes;
use std::path::Path;

use crate::sftp::SftpConfig;

pub fn upload_file_sftp(local_path: &str, remote_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = SftpConfig::from_env();
    
    // Connect to VM
    let tcp = std::net::TcpStream::connect(format!("{}:{}", cfg.host, cfg.port))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Authenticate
    session.userauth_pubkey_memory(
        &cfg.username,
        None,
        &cfg.private_key,
        cfg.passphrase.as_deref(),
    )?;
    assert!(session.authenticated());

    // Open SFTP session
    let sftp = session.sftp()?;

    // Open local file
    let mut local_file = std::fs::File::open(local_path)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;

    // Write to remote
    let mut remote_file = sftp.create(Path::new(remote_path))?;
    remote_file.write_all(&buffer)?;

    Ok(())
}

pub async fn stream_song_sftp(song_path: String) -> HttpResponse {
    let cfg = SftpConfig::from_env();

    // Connect to VM
    let tcp = std::net::TcpStream::connect(format!("{}:{}", cfg.host, cfg.port));
    if tcp.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    let tcp = tcp.unwrap();

    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();

    // Authenticate
    session.userauth_pubkey_memory(
        &cfg.username,
        None,
        &cfg.private_key,
        cfg.passphrase.as_deref(),
    ).unwrap();
    assert!(session.authenticated());

    let sftp = session.sftp().unwrap();
    let mut remote_file = match sftp.open(&song_path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let stream = async_stream::stream! {
        let mut buf = [0; 8192];
        loop {
            match remote_file.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => yield Ok::<_, std::io::Error>(Bytes::copy_from_slice(&buf[..n])),
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    };

    HttpResponse::Ok()
        .content_type(
            match Path::new(&song_path).extension().and_then(|ext| ext.to_str()) {
                Some("mp3") => "audio/mpeg",
                Some("wav") => "audio/wav",
                Some("flac") => "audio/flac",
                Some("ogg") => "audio/ogg",
                Some("aac") => "audio/aac",
                _ => "application/octet-stream",
            }
        )
        .streaming(stream)
}


pub fn delete_file_sftp(remote_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = SftpConfig::from_env();
    
    let tcp = std::net::TcpStream::connect(format!("{}:{}", cfg.host, cfg.port))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Authenticate
    session.userauth_pubkey_memory(
        &cfg.username,
        None,
        &cfg.private_key,
        cfg.passphrase.as_deref(),
    )?;
    assert!(session.authenticated());

    let sftp = session.sftp()?;
    sftp.unlink(Path::new(remote_path))?;

    Ok(())
}