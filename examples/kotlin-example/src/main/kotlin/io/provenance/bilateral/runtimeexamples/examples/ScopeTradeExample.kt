package io.provenance.bilateral.runtimeexamples.examples

import cosmos.tx.v1beta1.ServiceOuterClass
import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.execute.ExecuteMatch
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkBid
import io.provenance.bilateral.runtimeexamples.extensions.checkZeroResponseCode
import io.provenance.bilateral.runtimeexamples.extensions.verifyAskAndBidAreDeleted
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.bilateral.runtimeexamples.functions.coin
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody
import io.provenance.metadata.v1.MsgWriteScopeRequest
import io.provenance.metadata.v1.Party
import io.provenance.metadata.v1.PartyType
import io.provenance.scope.util.MetadataAddress
import io.provenance.scope.util.toByteString
import io.provenance.spec.HELOCSpecification
import java.time.OffsetDateTime
import java.util.UUID

object ScopeTradeExample : Example {
    override fun start(deps: AppDependencies) {
        val writeScopeSpec = deps.config.scopeTradeConfig.writeScopeSpec
        val client = deps.client
        val scopeUuid = UUID.randomUUID()
        if (writeScopeSpec) {
            val writeSpecMsgs = HELOCSpecification.specificationMsgs(deps.accounts.adminAccount.address())
            println("Writing HELOC asset type specification messages, owned by the contract admin: ${deps.accounts.adminAccount.address()}")
            client.pbClient.estimateAndBroadcastTx(
                txBody = writeSpecMsgs.map { it.toAny() }.toTxBody(),
                signers = deps.accounts.adminAccount.let(::BaseReqSigner).wrapList(),
                mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK,
                gasAdjustment = 1.3,
            ).checkZeroResponseCode()
        }

        val writeScopeMsg = MsgWriteScopeRequest.newBuilder().also { req ->
            req.scopeUuid  = scopeUuid.toString()
            req.specUuid = HELOCSpecification.scopeSpecConfig.id.toString()
            req.addSigners(deps.accounts.askerAccount.address())
            req.scopeBuilder.scopeId = MetadataAddress.forScope(scopeUuid).bytes.toByteString()
            req.scopeBuilder.specificationId = MetadataAddress.forScopeSpecification(HELOCSpecification.scopeSpecConfig.id).bytes.toByteString()
            req.scopeBuilder.valueOwnerAddress = client.contractAddress
            req.scopeBuilder.addOwners(Party.newBuilder().also { party ->
                party.address = client.contractAddress
                party.role = PartyType.PARTY_TYPE_OWNER
            })
            req.scopeBuilder.addDataAccess(deps.accounts.askerAccount.address())
        }.build().toAny()
        println("Creating scope with UUID [$scopeUuid] owned by the contract or whatever")
        client.pbClient.estimateAndBroadcastTx(
            txBody = writeScopeMsg.toTxBody(),
            signers = deps.accounts.askerAccount.let(::BaseReqSigner).wrapList(),
            mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK,
        ).checkZeroResponseCode()
        val askUuid = UUID.randomUUID()
        val createAsk = CreateAsk.newScopeTrade(
            id = askUuid.toString(),
            scopeAddress = MetadataAddress.forScope(scopeUuid).toString(),
            quote = coin(50000, "nhash").wrapList(),
            descriptor = RequestDescriptor("Example description", OffsetDateTime.now()),
        )
        println("Creating scope trade ask [$askUuid]")
        client.executeContract(signer = deps.accounts.askerAccount, executeMsg = createAsk,)
        client.checkAsk(askUuid)
        val bidUuid = UUID.randomUUID()
        val createBid = CreateBid.newScopeTrade(
            id = bidUuid.toString(),
            scopeAddress = MetadataAddress.forScope(scopeUuid).toString(),
            descriptor = RequestDescriptor("Example description", OffsetDateTime.now()),
        )
        println("Creating scope trade bid [$bidUuid]")
        client.executeContract(
            signer = deps.accounts.bidderAccount,
            executeMsg = createBid,
            funds = coin(50000, "nhash").wrapList(),
        )
        client.checkBid(bidUuid)
        val executeMatch = ExecuteMatch.new(askUuid.toString(), bidUuid.toString())
        client.executeContract(deps.accounts.adminAccount, executeMatch)
        client.verifyAskAndBidAreDeleted(askUuid, bidUuid)
        println("Successfully traded a scope")
    }
}
