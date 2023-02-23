use std::{
    sync::{
        atomic::{AtomicUsize, Ordering, AtomicU8},
        Arc,
    },
    time::{Duration, SystemTime},
};

use bytes::{Bytes, BytesMut};

use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    data_channel::{data_channel_init::RTCDataChannelInit, RTCDataChannel},
    ice_transport::{ice_candidate::RTCIceCandidate, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        RTCPeerConnection,
    },
};

use crate::config::MAX_DC_BUFFER_SIZE;

// const CHUNK_SIZE: usize = 64 * 1000; // 64 KB
const CHUNK_SIZE: usize = 32 * 1024;
// const BUFFERED_AMOUNT_LOW_THRESHOLD: usize = 1 * 1024 * 1024; 
const MAX_BUFFERED_AMOUNT: usize = 1 * 1024 * 1024; 
const BUFFERED_AMOUNT_LOW_THRESHOLD: usize = 256 * 1024; 
// const MAX_BUFFERED_AMOUNT: usize = 512 * 1024; // for some reason, rust can do this comfortably, but lots of channel activity

struct Dummy {
    id: usize,
    _dc: Option<Arc<RTCDataChannel>>,
}

impl Drop for Dummy {
    fn drop(&mut self) {
        println!("DROPPED DUMMY {}", self.id)
    }
}

pub async fn run_webrtc_rs_flow_control_test() -> anyhow::Result<()> {
    let (requester, requester_dc) = create_requester().await?;
    let responder = Arc::new(create_responder().await?);

    let requester_clone = Arc::clone(&requester);
    let requester_dc_clone = Arc::downgrade(&requester_dc);
    requester_dc.on_close(Box::new(move || { // FnMut can still move ownership! What makes it FnMut is how you end up using it
        let requester_clone1 = Arc::clone(&requester_clone);
        let requester_dc_clone1 = requester_dc_clone.clone();
        Box::pin(async move {
            // NOTE: on_close only outputs correct unsent_sctp_buffers is RESPONDER closes first
            // If requester closes first, you need to stop the sending first, then get the buffer, then close.
                // You will get some duplicates within the time from get_unsent_... and .close
                // But if you garauntee stop messaging before that, you will never get missed messages

            // on close doesn't fire sometimes??????

            let unsent_buffer = requester_clone1._HACK_get_unsent_sctp_buffers().await;
            println!("On close {}", unsent_buffer.len());
            if let Some(dc) = requester_dc_clone1.upgrade() {
                println!("DC Buffered Amount {}", dc.buffered_amount().await);
            }
        })
    }));

    let maybe_requester = Arc::downgrade(&requester);
    responder.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
        let maybe_requester = maybe_requester.clone();

        Box::pin(async move {
            if let Some(candidate) = candidate {
                if let Ok(candidate) = candidate.to_json() {
                    if let Some(requester) = maybe_requester.upgrade() {
                        if let Err(err) = requester.add_ice_candidate(candidate).await {
                            log::warn!("{}", err);
                        }
                    }
                }
            }
        })
    }));

    let maybe_responder = Arc::downgrade(&responder);
    requester.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
        let maybe_responder = maybe_responder.clone();

        Box::pin(async move {
            if let Some(candidate) = candidate {
                if let Ok(candidate) = candidate.to_json() {
                    if let Some(responder) = maybe_responder.upgrade() {
                        if let Err(err) = responder.add_ice_candidate(candidate).await {
                            log::warn!("{}", err);
                        }
                    }
                }
            }
        })
    }));

    let (fault, mut reqs_fault) = tokio::sync::mpsc::channel(1);
    requester.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        let fault = fault.clone();

        Box::pin(async move {
            println!("Requester {:?}", s);
            if s == RTCPeerConnectionState::Failed {
                fault.send(()).await.unwrap();
            }
        })
    }));

    let (fault, mut resp_fault) = tokio::sync::mpsc::channel(1);
    responder.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        let fault = fault.clone();

        Box::pin(async move {
            println!("Responder {:?}", s);
            if s == RTCPeerConnectionState::Failed {
                fault.send(()).await.unwrap();
            }
        })
    }));

    let reqs = requester.create_offer(None).await?;

    requester.set_local_description(reqs.clone()).await?;
    responder.set_remote_description(reqs).await?;

    let resp = responder.create_answer(None).await?;

    responder.set_local_description(resp.clone()).await?;
    requester.set_remote_description(resp).await?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = reqs_fault.recv() => {
            log::error!("Requester's peer connection failed...")
        }
        _ = resp_fault.recv() => {
            log::error!("Responder's peer connection failed...");
        }
    }

    println!();
    let buffered_amount = requester_dc.buffered_amount().await;
    let unsent_buffer = requester._HACK_get_unsent_sctp_buffers().await;
    println!("Before close {}", unsent_buffer.len());
    println!("DC Buffered Amount {}", buffered_amount);

    // requester.close().await?;
    responder.close().await?;

    // let mut bytes: bytes::BytesMut = BytesMut::with_capacity(buffered_amount);
    // for b in unsent_buffer {
    //     bytes.extend_from_slice(&b);
    // }

    // println!("{}", bytes.len());
    // for b in bytes {
    //     if b != 0 {
    //         println!("{}", b);
    //     }
    // }

    // Outputs 0
    // let unsent_buffer = requester._HACK_get_unsent_sctp_buffers().await;
    // println!("{}", unsent_buffer.len());

    println!();

    Ok(())
}

async fn create_peer_connection() -> anyhow::Result<RTCPeerConnection> {
    // Create unique MediaEngine,
    // as MediaEngine must not be shared between PeerConnections
    let mut media_engine = MediaEngine::default();

    media_engine.register_default_codecs()?;

    let mut interceptor_registry = Registry::new();

    interceptor_registry = register_default_interceptors(interceptor_registry, &mut media_engine)?;

    // Create API that bundles the global functions of the WebRTC API
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(interceptor_registry)
        .build();

    let ice_servers = vec![RTCIceServer {
        ..Default::default()
    }];

    let config = RTCConfiguration {
        ice_servers,
        ..Default::default()
    };

    Ok(api.new_peer_connection(config).await?)
}

async fn create_requester() -> anyhow::Result<(Arc<RTCPeerConnection>, Arc<RTCDataChannel>)> {
    // Create a peer connection first
    let pc = Arc::new(create_peer_connection().await?);

    pc.on_signaling_state_change(Box::new(|state| {
        Box::pin(async move {
            println!("Signaling state change REQUESTER {:?}", state);
        })
    }));

    pc.on_peer_connection_state_change(Box::new(|state| {
        Box::pin(async move {
            println!("PC state change REQUESTER {:?}", state);
        })
    }));

    // Data transmission requires a data channel, so prepare to create one
    let options = Some(RTCDataChannelInit {
        ordered: Some(false),
        max_retransmits: Some(0u16),
        ..Default::default()
    });

    // Create a data channel to send data over a peer connection
    let dc = pc.create_data_channel("data", options).await?;

    // Use mpsc channel to send and receive a signal when more data can be sent
    let (more_can_be_sent, mut maybe_more_can_be_sent) = tokio::sync::mpsc::channel(1);

    // Get a shared pointer to the data channel
    let shared_dc = dc.clone();
    dc.on_open(Box::new(|| {
        Box::pin(async move {
            let counter = AtomicU8::new(0);
            // This callback shouldn't be blocked for a long time, so we spawn our handler
            tokio::spawn(async move {
                // let buf = Bytes::from_static(&[0u8; CHUNK_SIZE]);
                loop {
                    let mut ary = [0u8; CHUNK_SIZE];
                    ary[0] = counter.fetch_add(1, Ordering::SeqCst);
                    // println!("{}", ary[0]);
                    let buf = Bytes::from(Box::from(ary));
                    if shared_dc.send(&buf).await.is_err() {
                        break;
                    }

                    let buffered_amount = shared_dc.buffered_amount().await;
                    // println!("Buffered Amount {}", buffered_amount);

                    if buffered_amount + buf.len() > MAX_BUFFERED_AMOUNT {
                        // Wait for the signal that more can be sent
                        let _ = maybe_more_can_be_sent.recv().await;
                    }
                }
            });
        })
    }));

    dc.set_buffered_amount_low_threshold(BUFFERED_AMOUNT_LOW_THRESHOLD)
        .await;

    dc.on_buffered_amount_low(Box::new(move || {
        let more_can_be_sent = more_can_be_sent.clone();
        Box::pin(async move {
            // Send a signal that more can be sent
            more_can_be_sent.send(()).await.unwrap();
        })
    }))
    .await;

    Ok((pc, dc))
}

async fn create_responder() -> anyhow::Result<RTCPeerConnection> {
    // Create a peer connection first
    let pc = create_peer_connection().await?;

    pc.on_signaling_state_change(Box::new(|state| {
        Box::pin(async move {
            println!("Signaling state change RESPONDER {:?}", state);
        })
    }));

    pc.on_peer_connection_state_change(Box::new(|state| {
        Box::pin(async move {
            println!("PC state change RESPONDER {:?}", state);
        })
    }));

    // let dummy_weak = Arc::downgrade(&dummy_arc);

    // Set a data channel handler so that we can receive data
    pc.on_data_channel(Box::new(move |dc| {
        Box::pin(async move {
            let total_bytes_received = Arc::new(AtomicUsize::new(0));
                    
            // MEMORY LEAK
            let dummy_arc = Arc::new(Dummy {
                id: 0,
                _dc: Some(dc.clone()),
                // dc: None,
            });

            let shared_total_bytes_received = total_bytes_received.clone();
            dc.on_open(Box::new(move || {
                Box::pin(async {
                    // This callback shouldn't be blocked for a long time, so we spawn our handler
                    tokio::spawn(async move {
                        let start = SystemTime::now();

                        tokio::time::sleep(Duration::from_secs(1)).await;
                        println!();

                        loop {
                            let total_bytes_received =
                                shared_total_bytes_received.load(Ordering::Relaxed);

                            let elapsed = SystemTime::now().duration_since(start);
                            let bps =
                                (total_bytes_received * 8) as f64 / elapsed.unwrap().as_secs_f64();

                            println!(
                                "Throughput is about {:.03} Mbps",
                                bps / (1024 * 1024) as f64
                            );
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    });
                })
            }));

            dc.on_message(Box::new(move |msg| {
                let total_bytes_received = total_bytes_received.clone();

                let _ = dummy_arc.as_ref();

                Box::pin(async move {
                    total_bytes_received.fetch_add(msg.data.len(), Ordering::Relaxed);
                    // tokio::time::sleep(Duration::from_millis(100)).await;
                })
            }));

            // let _ = dummy_arc.as_ref();
        })
    }));

    Ok(pc)
}

// let options = Some(RTCDataChannelInit {
//     negotiated: Some(2),
//     ordered: Some(false),
//     max_retransmits: Some(0u16),
//     ..Default::default()
// });

// // Create a data channel to send data over a peer connection
// let dc = connection.create_data_channel("data", options).await?;

pub async fn attatch_requester_dc(dc: &Arc<RTCDataChannel>) -> anyhow::Result<()> {
    // Use mpsc channel to send and receive a signal when more data can be sent
    let (more_can_be_sent, mut maybe_more_can_be_sent) = tokio::sync::mpsc::channel(1);

    // Get a shared pointer to the data channel
    let shared_dc = dc.clone();
    dc.on_open(Box::new(|| {
        Box::pin(async move {
            let counter = std::sync::atomic::AtomicU8::new(0);
            // This callback shouldn't be blocked for a long time, so we spawn our handler
            tokio::spawn(async move {
                loop {
                    let mut ary = [0u8; CHUNK_SIZE];
                    ary[0] = counter.fetch_add(1, Ordering::SeqCst);
                    // println!("{}", ary[0]);
                    let buf = Bytes::from(Box::from(ary));
                    if shared_dc.send(&buf).await.is_err() {
                        break;
                    }

                    let buffered_amount = shared_dc.buffered_amount().await;
                    // println!("Buffered Amount {}", buffered_amount);

                    if buffered_amount + buf.len() > MAX_DC_BUFFER_SIZE {
                        // Wait for the signal that more can be sent
                        let _ = maybe_more_can_be_sent.recv().await;
                    }
                }
            });
        })
    }));

    dc.set_buffered_amount_low_threshold(BUFFERED_AMOUNT_LOW_THRESHOLD)
        .await;

    dc.on_buffered_amount_low(Box::new(move || {
        let more_can_be_sent = more_can_be_sent.clone();
        Box::pin(async move {
            // Send a signal that more can be sent
            more_can_be_sent.send(()).await.unwrap();
        })
    }))
    .await;

    anyhow::Ok(())
}

pub async fn attatch_responder_dc(dc: &RTCDataChannel) -> anyhow::Result<()> {
    let total_bytes_received = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let shared_total_bytes_received = total_bytes_received.clone();
    dc.on_open(Box::new(move || {
        Box::pin(async {
            // This callback shouldn't be blocked for a long time, so we spawn our handler
            tokio::spawn(async move {
                let start = std::time::SystemTime::now();

                tokio::time::sleep(Duration::from_secs(1)).await;
                println!();

                loop {
                    let total_bytes_received =
                        shared_total_bytes_received.load(Ordering::Relaxed);

                    let elapsed = std::time::SystemTime::now().duration_since(start);
                    let bps =
                        (total_bytes_received * 8) as f64 / elapsed.unwrap().as_secs_f64();

                    println!(
                        "Throughput is about {:.03} Mbps",
                        bps / (1024 * 1024) as f64
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
        })
    }));

    dc.on_message(Box::new(move |msg| {
        let total_bytes_received = total_bytes_received.clone();

        Box::pin(async move {
            total_bytes_received.fetch_add(msg.data.len(), Ordering::Relaxed);
        })
    }));

    anyhow::Ok(())
}
