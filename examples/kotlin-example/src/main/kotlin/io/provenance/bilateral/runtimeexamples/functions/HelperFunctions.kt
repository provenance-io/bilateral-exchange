package io.provenance.bilateral.runtimeexamples.functions

import cosmos.base.v1beta1.CoinOuterClass.Coin
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastMode
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.extensions.checkZeroResponseCode
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.Signer
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody
import io.provenance.marker.v1.Access
import io.provenance.marker.v1.AccessGrant
import io.provenance.marker.v1.MarkerStatus
import io.provenance.marker.v1.MarkerType
import io.provenance.marker.v1.MsgActivateRequest
import io.provenance.marker.v1.MsgAddAccessRequest
import io.provenance.marker.v1.MsgAddMarkerRequest
import io.provenance.name.v1.MsgBindNameRequest
import io.provenance.name.v1.NameRecord

fun coin(amount: Long, denom: String): Coin = Coin.newBuilder().setAmount(amount.toString()).setDenom(denom).build()

fun coins(amount: Long, denom: String): List<Coin> = listOf(coin(amount, denom))

fun createMarker(
    deps: AppDependencies,
    ownerAccount: Signer,
    denomName: String,
    supply: Long,
    fixed: Boolean = true,
    allowGovControl: Boolean = true,
) {
    val addReq = MsgAddMarkerRequest.newBuilder().also { req ->
        req.amount = coin(supply, denomName)
        req.fromAddress = ownerAccount.address()
        req.markerType = MarkerType.MARKER_TYPE_COIN
        req.status = MarkerStatus.MARKER_STATUS_FINALIZED
        req.supplyFixed = fixed
        req.allowGovernanceControl = allowGovControl
        req.addAccessList(AccessGrant.newBuilder().also { grant ->
            grant.address = ownerAccount.address()
            // Mimics the grants given in asset manager
            grant.addAllPermissions(
                listOf(
                    Access.ACCESS_ADMIN,
                    Access.ACCESS_DEPOSIT,
                    Access.ACCESS_WITHDRAW,
                    Access.ACCESS_BURN,
                    Access.ACCESS_MINT,
                    Access.ACCESS_DELETE,
                )
            )
        })
    }.build()
    val activateReq = MsgActivateRequest.newBuilder().also { req ->
        req.administrator = ownerAccount.address()
        req.denom = denomName
    }.build()
    println("Creating marker of denom [$denomName] for admin account [${ownerAccount.address()}]")
    deps.client.pbClient.estimateAndBroadcastTx(
        txBody = listOf(addReq, activateReq).map { it.toAny() }.toTxBody(),
        signers = ownerAccount.let(::BaseReqSigner).wrapList(),
        mode = BroadcastMode.BROADCAST_MODE_BLOCK,
        gasAdjustment = 1.3,
    ).checkZeroResponseCode()
}

fun grantMarkerAccess(
    deps: AppDependencies,
    markerAdminAccount: Signer,
    markerDenom: String,
    grantAddress: String,
    permissions: List<Access> = listOf(Access.ACCESS_ADMIN),
) {
    val accessReq = MsgAddAccessRequest.newBuilder().also { req ->
        req.denom = markerDenom
        req.administrator = markerAdminAccount.address()
        req.addAccess(AccessGrant.newBuilder().also { grant ->
            grant.address = grantAddress
            grant.addAllPermissions(permissions)
        })
    }.build()
    println("Granting permissions $permissions to address [$grantAddress] for marker with denom [$markerDenom]")
    deps.client.pbClient.estimateAndBroadcastTx(
        txBody = accessReq.toAny().toTxBody(),
        signers = markerAdminAccount.let(::BaseReqSigner).wrapList(),
        mode = BroadcastMode.BROADCAST_MODE_BLOCK,
        gasAdjustment = 1.3,
    ).checkZeroResponseCode()
}

fun bindNamesToSigner(
    deps: AppDependencies,
    names: List<String>,
    signer: Signer,
    restricted: Boolean,
) {
    names.map { name ->
        MsgBindNameRequest.newBuilder().also { bindName ->
            val nameParts = name.split(".")
            val rootName = nameParts.first()
            val parentName = nameParts.drop(1).joinToString("")
            println("Building name binding for [${signer.address()}] with root name [$rootName] to parent name [$parentName] for combined name [$name]")
            bindName.parent = NameRecord.newBuilder().also { record ->
                record.name = parentName
                record.address = signer.address()
            }.build()
            bindName.record = NameRecord.newBuilder().also { record ->
                record.name = rootName
                record.address = signer.address()
                record.restricted = restricted
            }.build()
        }.build().toAny()
    }.also { nameMsgs ->
        println("Sending transaction to bind names $names to signer [${signer.address()}]")
        deps.client.pbClient.estimateAndBroadcastTx(
            txBody = nameMsgs.toTxBody(),
            signers = BaseReqSigner(signer).wrapList(),
            mode = BroadcastMode.BROADCAST_MODE_BLOCK,
            gasAdjustment = 1.3,
        ).checkZeroResponseCode().also {
            println("Successfully bound names $names to signer [${signer.address()}]")
        }
    }
}
