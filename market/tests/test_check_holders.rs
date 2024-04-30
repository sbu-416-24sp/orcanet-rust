use std::{net::Ipv4Addr, time::Duration};

use libp2p::Multiaddr;
use orcanet_market::{
    bridge::spawn, BootNodes, Config, FileInfoHash, FileResponse, Protocol,
    ReqResSuccessfulResponse, SuccessfulResponse, SupplierInfo,
};
use proto::market::{FileInfo, HoldersResponse, User};

#[tokio::test]
async fn test_register_file_and_get_self_holder() {
    let config = Config::builder().set_peer_tcp_port(3390).build();
    let peer = spawn(config).unwrap();
    let peer_id = *peer.peer_id();
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
    let expected_holder = SupplierInfo {
        file_info: file_info.clone(),
        user: user.clone(),
    };
    let file_info_hash = FileInfoHash::new(file_info.hash_to_string());
    let _ = peer
        .register_file(user, file_info_hash.clone(), file_info)
        .await;
    let res = peer.get_holder_by_peer_id(peer_id, file_info_hash).await;
    assert_eq!(
        res,
        Ok(SuccessfulResponse::ReqResResponse(
            ReqResSuccessfulResponse::GetHolderByPeerId {
                holder: FileResponse::HasFile(expected_holder)
            }
        ))
    )
}

#[tokio::test]
async fn test_register_file_and_check_holders_basic() {
    let config = Config::builder().set_peer_tcp_port(3391).build();
    let peer = spawn(config).unwrap();
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
    let expected_holders = HoldersResponse {
        file_info: Some(file_info.clone()),
        holders: vec![user.clone()],
    };
    let file_info_hash = FileInfoHash::new(file_info.hash_to_string());
    let _ = peer
        .register_file(user, file_info_hash.clone(), file_info)
        .await;
    let res = peer.check_holders(file_info_hash).await;
    assert_eq!(res, Ok(SuccessfulResponse::CheckHolders(expected_holders)))
}

#[tokio::test]
async fn test_check_holders_from_other_peer() {
    let config = Config::builder().set_peer_tcp_port(3392).build();
    let peer1 = spawn(config).unwrap();
    let mut addr = Multiaddr::empty();
    addr.push(Protocol::Ip4(Ipv4Addr::LOCALHOST));
    addr.push(Protocol::Tcp(3392));
    addr.push(Protocol::P2p(*peer1.peer_id()));

    let boot_nodes = BootNodes::with_nodes(vec![addr]);
    let config = Config::builder()
        .set_peer_tcp_port(3393)
        .set_boot_nodes(boot_nodes)
        .build();
    let peer2 = spawn(config).unwrap();
    // give it a bit to connect
    tokio::time::sleep(Duration::from_secs(1)).await;

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
    let expected_holders = HoldersResponse {
        file_info: Some(file_info.clone()),
        holders: vec![user.clone()],
    };

    let file_info_hash = FileInfoHash::new(file_info.hash_to_string());
    let _ = peer1
        .register_file(user, file_info_hash.clone(), file_info)
        .await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let res = peer2.check_holders(file_info_hash).await;
    assert_eq!(res, Ok(SuccessfulResponse::CheckHolders(expected_holders)))
}
