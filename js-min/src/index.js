/* eslint-disable no-console */

/*
 * Dialer Node
 */

import { createLibp2p } from "libp2p";
import { pipe } from "it-pipe";
import { createFromJSON } from "@libp2p/peer-id-factory";
import { toString as uint8ArrayToString } from "uint8arrays/to-string";
import { fromString as uint8ArrayFromString } from "uint8arrays/from-string";
import { multiaddr } from "@multiformats/multiaddr";
import { createEd25519PeerId } from "@libp2p/peer-id-factory";
import { noise } from "@chainsafe/libp2p-noise";
import { mplex } from "@libp2p/mplex";
import { webSockets } from "@libp2p/websockets";
import { all } from "@libp2p/websockets/filters";

export const PROTOCOL_NAME = "/fluence/particle/2.0.0";
export const RUST_NODE_ADDRESS =
  "/ip4/127.0.0.1/tcp/4310/ws/p2p/12D3KooWEc4rWf38ZHDqbArm1vFiVwc9wDooBdmbGhwzRRhsZnEN";

async function run() {
  // Dialer
  const dialerNode = await createLibp2p({
    peerId: await createEd25519PeerId(),
    transports: [
      webSockets({
        filter: all,
      }),
    ],
    streamMuxers: [mplex()],
    connectionEncryption: [noise()],
  });

  // Add peer to Dial (the listener) into the PeerStore
  const listenerMultiaddr = multiaddr(RUST_NODE_ADDRESS);

  console.log("Dialer ready, listening on:");

  // Dial the listener node
  console.log("Dialing to peer:", listenerMultiaddr);
  const stream = await dialerNode.dialProtocol(
    listenerMultiaddr,
    PROTOCOL_NAME
  );

  console.log("nodeA dialed to nodeB on protocol: " + PROTOCOL_NAME);

  pipe(
    // Source data
    [uint8ArrayFromString("hey")],
    // Write to the stream, and pass its output to the next function
    stream,
    // Sink function
    async function (source) {
      // For each chunk of data
      for await (const data of source) {
        // Output the data
        console.log("received echo:", uint8ArrayToString(data.subarray()));
      }
    }
  );
}

run();
