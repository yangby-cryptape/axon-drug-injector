/* Required Environment Variables:
 * - JSONRPC_URL
 * - INJECTOR_JSONRPC_URL
 * - PRIVATE_KEY
 */

const axios = require("axios");
const ethers = require("ethers");

async function tryBroadcastTransaction(url, signedTx): Promise<Record<string>> {
  const data = {
    jsonrpc: '2.0',
    method: 'eth_sendRawTransaction',
    params: [signedTx],
    id: 1
  };
  const options = {
    headers: {
      'Content-Type': 'application/json'
    }
  };
  const response = await axios.post(url, data, options);
  return response.data.result;
}

async function main() {
  const defaultUrl = process.env.JSONRPC_URL
  const provider = ethers.getDefaultProvider(defaultUrl);

  const privateKey = process.env.PRIVATE_KEY;
  const sender = new ethers.Wallet(privateKey, provider);
  console.log(`>>> sender   address: ${sender.address}`);
  const receiver = sender;

  const nonce = await provider.getTransactionCount(sender.address);
  console.log(`>>> nonce: ${nonce}`);
  const network = await provider.getNetwork();
  console.log(`>>> chain-id: ${network.chainId}`);

  let unsignedRawTx = new ethers.Transaction();
  unsignedRawTx.chainId = network.chainId;
  unsignedRawTx.to = receiver.address;
  unsignedRawTx.nonce = nonce;
  unsignedRawTx.type = 0;
  unsignedRawTx.value = ethers.parseEther("1");
  console.log(">>> unsigned-raw-tx:");
  console.log(unsignedRawTx);
  console.log(">>> ====    ====    ====    ====");

  let unsignedTx = await sender.populateTransaction(unsignedRawTx);
  unsignedTx.gasPrice = "0x1";
  unsignedTx.gasLimit = "0x10000000000000000";

  console.log(">>> unsigned-tx:");
  console.log(unsignedTx);
  console.log(">>> ====    ====    ====    ====");
  let signedTx = await sender.signTransaction(unsignedTx);
  console.log(">>> signed-tx:");
  console.log(signedTx);
  console.log(">>> ====    ====    ====    ====");

  const injectorUrl = process.env.INJECTOR_JSONRPC_URL
  var response = await tryBroadcastTransaction(injectorUrl, signedTx);
  console.log(`P2P Broadcast: ${response}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
