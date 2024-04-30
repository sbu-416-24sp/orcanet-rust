use orcanet_market::{
    bridge::spawn, Config, FileInfoHash, KadSuccessfulResponse, SuccessfulResponse,
};
use proto::market::{FileInfo, User};

#[tokio::test]
async fn test_register_file() {
    let peer = spawn(Config::default()).unwrap();
    let user = User {
        id: "abc".to_string(),
        name: "helloworld".to_string(),
        ip: "127.0.0.1".to_string(),
        port: 6666,
        price: 32,
    };
    let file_info = FileInfo {
        file_hash: "123abc".to_string(),
        chunk_hashes: vec!["hi".to_string()],
        file_size: 3212321,
        file_name: "fooobar.mp4".to_owned(),
    };
    let file_info_hash = FileInfoHash::new(file_info.hash_to_string());
    let res = peer.register_file(user, file_info_hash, file_info).await;
    assert_eq!(
        res,
        Ok(SuccessfulResponse::KadResponse(
            KadSuccessfulResponse::RegisterFile
        ))
    )
}

#[tokio::test]
async fn test_register_and_get_providers_for_one() {
    let config = Config::builder().set_peer_tcp_port(14000).build();
    let peer = spawn(config).unwrap();
    let peer_id = peer.peer_id();
    let user = User {
        id: "abc".to_string(),
        name: "helloworld".to_string(),
        ip: "127.0.0.1".to_string(),
        port: 6666,
        price: 32,
    };
    let file_info = FileInfo {
        file_hash: "123abc".to_string(),
        chunk_hashes: vec!["hi".to_string()],
        file_size: 3212321,
        file_name: "fooobar.mp4".to_owned(),
    };
    let file_info_hash = FileInfoHash::new(file_info.hash_to_string());
    let _ = peer
        .register_file(user, file_info_hash.clone(), file_info)
        .await;
    let res = peer.get_providers(file_info_hash).await;
    let expected_providers = vec![*peer_id];
    assert_eq!(
        res,
        Ok(SuccessfulResponse::KadResponse(
            KadSuccessfulResponse::GetProviders {
                providers: expected_providers
            }
        ))
    );
}
