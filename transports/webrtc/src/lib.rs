// Copyright 2022 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Implementation of the [`libp2p_core::Transport`] trait for WebRTC protocol without a signaling
//! server.
//!
//! # Overview
//!
//! ## ICE
//!
//! RFCs: 8839, 8445 See also:
//! <https://tools.ietf.org/id/draft-ietf-rtcweb-sdp-08.html#rfc.section.5.2.3>
//!
//! The WebRTC protocol uses ICE in order to establish a connection.
//!
//! In a typical ICE setup, there are two endpoints, called agents, that want to communicate. One
//! of these two agents can be the local browser, while the other agent is the target of the
//! connection.
//!
//! Even though in this specific context all we want is a simple client-server communication, it is
//! helpful to keep in mind that ICE was designed to solve the problem of NAT traversal.
//!
//! The ICE workflow works as follows:
//!
//! - An "offerer" determines ways in which it could be accessible (either an
//!   IP address or through a relay using a TURN server), which are called "candidates". It then
//!   generates a small text payload in a format called SDP, that describes the request for a
//!   connection.
//! - The offerer sends this SDP-encoded message to the answerer. The medium through which this
//!   exchange is done is out of scope of the ICE protocol.
//! - The answerer then finds its own candidates, and generates an answer, again in the SDP format.
//!   This answer is sent back to the offerer.
//! - Each agent then tries to connect to the remote's candidates.
//!
//! We pretend to send the offer to the remote agent (the target of the connection), then pretend
//! that it has found a valid IP address for itself (i.e. a candidate), then pretend that the SDP
//! answer containing this candidate has been sent back. This will cause the offerer to execute
//! step 4: try to connect to the remote's candidate.
//!
//! ## TCP or UDP
//!
//! WebRTC by itself doesn't hardcode any specific protocol for media streams. Instead, it is the
//! SDP message of the offerer that specifies which protocol to use. In our use case (one or more
//! data channels), we know that the offerer will always request either TCP+DTLS+SCTP, or
//! UDP+DTLS+SCTP.
//!
//! The implementation only supports UDP at the moment, so if the offerer requests TCP+DTLS+SCTP, it
//! will not respond. Support for TCP may be added in the future (see
//! <https://github.com/webrtc-rs/webrtc/issues/132>).
//!
//! ## DTLS+SCTP
//!
//! RFCs: 8841, 8832
//!
//! In both cases (TCP or UDP), the next layer is DTLS. DTLS is similar to the well-known TLS
//! protocol, except that it doesn't guarantee ordering of delivery (as this is instead provided by
//! the SCTP layer on top of DTLS). In other words, once the TCP or UDP connection is established,
//! the browser will try to perform a DTLS handshake.
//!
//! During the ICE negotiation, each agent must include in its SDP packet a hash of the self-signed
//! certificate that it will use during the DTLS handshake. In our use-case, where we try to
//! hand-crate the SDP answer generated by the remote, this is problematic. A way to solve this
//! is to make the hash a part of the remote's multiaddr. On the server side, we turn
//! certificate verification off.

mod proto {
    #![allow(unreachable_pub)]
    include!("generated/mod.rs");
    pub(crate) use self::webrtc::pb::{mod_Message::Flag, Message};
}

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "wasm-bindgen")]
pub mod websys;

/// Crates common to all features
mod common;
