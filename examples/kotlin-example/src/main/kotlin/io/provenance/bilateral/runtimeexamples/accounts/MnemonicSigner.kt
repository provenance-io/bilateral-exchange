package io.provenance.bilateral.runtimeexamples.accounts

import com.google.protobuf.ByteString
import cosmos.crypto.secp256k1.Keys
import io.provenance.client.grpc.Signer
import io.provenance.hdwallet.bip39.MnemonicWords
import io.provenance.hdwallet.common.hashing.sha256
import io.provenance.hdwallet.ec.ECKeyPair
import io.provenance.hdwallet.hrp.Hrp
import io.provenance.hdwallet.signer.BCECSigner
import io.provenance.hdwallet.wallet.Wallet

data class MnemonicSigner(
    val mnemonic: String,
    private val address: String,
    private val keyPair: ECKeyPair,
) : Signer {
    companion object {
        fun new(
            mnemonic: String,
            hdPath: String = "m/44'/1'/0'/0/0'"
        ): MnemonicSigner = Wallet.fromMnemonic(
            hrp = Hrp.ProvenanceBlockchain.testnet,
            passphrase = "",
            mnemonicWords = MnemonicWords.of(mnemonic),
            testnet = true,
        )[hdPath].let { account ->
            MnemonicSigner(
                mnemonic = mnemonic,
                address = account.address.value,
                keyPair = account.keyPair,
            )
        }
    }

    override fun address(): String = address

    override fun pubKey(): Keys.PubKey = Keys.PubKey.newBuilder().setKey(ByteString.copyFrom(keyPair.publicKey.compressed())).build()

    override fun sign(data: ByteArray): ByteArray = BCECSigner().sign(keyPair.privateKey, data.sha256()).encodeAsBTC().toByteArray()
}
