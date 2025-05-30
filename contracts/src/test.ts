import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers"
import { AbstractSigner, BaseWallet, Mnemonic, SigningKey, Transaction, TransactionLike, TransactionRequest, Wallet, ZeroAddress } from "ethers"
import hre from "hardhat"

async function main() {
  // const [signer] = await hre.ethers.getSigners()
  const wallet =
    // new BaseWallet(new SigningKey("0xb03ddbc95b42380ce8c1dee760d14bd84881750a50817b33852791b0d1b30ccf"));
    Wallet.fromPhrase("test test test test test test test test test test test junk");

  const latestBlock = await hre.ethers.provider.getBlock(await hre.ethers.provider.getBlockNumber())
  if (!latestBlock)
    throw new Error("cant get latest block");
  const factory = await hre.ethers.getContractFactory("Lock")
  const deployTx: TransactionLike = await factory.getDeployTransaction(latestBlock.timestamp + 20_000);
  const tx: TransactionLike = {
    chainId: 31337,
    nonce: 0,
    gasLimit: 2_000_000,
    maxFeePerGas: 875_000_000,
    maxPriorityFeePerGas: 10_000,
    // to: TxKind::Create,
    to: ZeroAddress,
    value: 0,
    accessList: [],
    authorizationList: [],
    data: factory.bytecode,
  };
  // console.log(await hre.ethers.provider.estimateGas(deployTx))
  console.log(tx)

  // const tx: TransactionLike = {
  //   to: '0xa238b6008Bc2FBd9E386A5d4784511980cE504Cd',
  //   value: hre.ethers.parseEther('1'),
  //   gasLimit: '21000',
  //   maxPriorityFeePerGas: hre.ethers.parseUnits('5', 'gwei'),
  //   maxFeePerGas: hre.ethers.parseUnits('20', 'gwei'),
  //   nonce: 1,
  //   type: 2,
  //   chainId: 3
  // }
  // const signed_raw_tx = await wallet.signTransaction(tx)
  // console.log("signed")
  // const signed_tx = Transaction.from(tx).serialized
  // console.log("serialized")

  // console.log(await hre.ethers.provider.send("eth_sendRawTransaction", [signed_raw_tx]))
}

main()
