use js_sys::Reflect;
use libp2p_webrtc_utils::fingerprint::Fingerprint;
use libp2p_webrtc_utils::sdp::render_description;
use std::net::SocketAddr;
use wasm_bindgen::JsValue;
use web_sys::{RtcSdpType, RtcSessionDescriptionInit};

/// Creates the SDP answer used by the client.
pub(crate) fn answer(
    addr: SocketAddr,
    server_fingerprint: &Fingerprint,
    client_ufrag: &str,
) -> RtcSessionDescriptionInit {
    let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.sdp(&render_description(
        SESSION_DESCRIPTION,
        addr,
        server_fingerprint,
        client_ufrag,
    ));
    answer_obj
}

/// Creates the munged SDP offer from the Browser's given SDP offer
///
/// Certificate verification is disabled which is why we hardcode a dummy fingerprint here.
pub(crate) fn offer(offer: JsValue, client_ufrag: &str) -> RtcSessionDescriptionInit {
    //JsValue to String
    let offer = Reflect::get(&offer, &JsValue::from_str("sdp")).unwrap();
    let offer = offer.as_string().unwrap();

    let lines = offer.split("\r\n");

    // find line and replace a=ice-ufrag: with "\r\na=ice-ufrag:{client_ufrag}\r\n"
    // find line andreplace a=ice-pwd: with "\r\na=ice-ufrag:{client_ufrag}\r\n"

    let mut munged_offer_sdp = String::new();

    for line in lines {
        if line.starts_with("a=ice-ufrag:") {
            munged_offer_sdp.push_str(&format!("a=ice-ufrag:{}\r\n", client_ufrag));
        } else if line.starts_with("a=ice-pwd:") {
            munged_offer_sdp.push_str(&format!("a=ice-pwd:{}\r\n", client_ufrag));
        } else if !line.is_empty() {
            munged_offer_sdp.push_str(&format!("{}\r\n", line));
        }
    }

    // remove any double \r\n
    let munged_offer_sdp = munged_offer_sdp.replace("\r\n\r\n", "\r\n");

    log::trace!("munged_offer_sdp: {}", munged_offer_sdp);

    // setLocalDescription
    let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.sdp(&munged_offer_sdp);

    offer_obj
}

const SESSION_DESCRIPTION: &str = "v=0
o=- 0 0 IN {ip_version} {target_ip}
s=-
c=IN {ip_version} {target_ip}
t=0 0
a=ice-lite
m=application {target_port} UDP/DTLS/SCTP webrtc-datachannel
a=mid:0
a=setup:passive
a=ice-ufrag:{ufrag}
a=ice-pwd:{pwd}
a=fingerprint:{fingerprint_algorithm} {fingerprint_value}
a=sctp-port:5000
a=max-message-size:16384
a=candidate:1467250027 1 UDP 1467250027 {target_ip} {target_port} typ host
";