package io.provenance.bilateral.runtimeexamples.examples

import cosmos.base.v1beta1.CoinOuterClass.Coin
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastMode
import io.provenance.attribute.v1.AttributeType
import io.provenance.attribute.v1.MsgAddAttributeRequest
import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.execute.ExecuteMatch
import io.provenance.bilateral.models.AttributeRequirement
import io.provenance.bilateral.models.AttributeRequirementType
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkBid
import io.provenance.bilateral.runtimeexamples.extensions.verifyAskAndBidAreDeleted
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.bilateral.runtimeexamples.functions.bindNamesToSigner
import io.provenance.bilateral.runtimeexamples.functions.coins
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.Signer
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody
import io.provenance.scope.util.toByteString
import java.time.OffsetDateTime
import java.util.UUID

object RequiredAttributesExample : Example {
    private val DEFAULT_QUOTE: List<Coin> = coins(100, "nhash")
    private val DEFAULT_BASE: List<Coin> = coins(100, "nhash")

    // All of these examples use the coin_trade ask and bid type for its simplicity, but these required attributes
    // can be used with any request type.  These tests assume the asker and bidder accounts do not include the specified
    // attributes in the local.toml file.  If the accounts already have the attributes, these functionalities will
    // likely behave strangely and/or outright fail.
    override fun start(deps: AppDependencies) {
        check(deps.config.requiredAttributesConfig.attributes.isNotEmpty()) { "No attributes were provided for the RequiredAttributesExample.  These examples need at least one value to properly operate" }
        val requirementType = deps.config.requiredAttributesConfig.getRequirementTypeEnum().also { type ->
            println("Running example of type [$type] with attributes ${deps.config.requiredAttributesConfig.attributes}")
        }
        when (requirementType) {
            AttributeRequirementType.ALL -> doAllExample(deps)
            AttributeRequirementType.ANY -> doAnyExample(deps)
            AttributeRequirementType.NONE -> doNoneExample(deps)
        }
    }

    private fun doAllExample(deps: AppDependencies) {
        val client = deps.client
        val attributes = deps.config.requiredAttributesConfig.attributes
        println("Binding all attribute names $attributes to admin address [${deps.accounts.adminAccount.address()}]")
        bindNamesToSigner(
            deps = deps,
            names = attributes,
            signer = deps.accounts.adminAccount,
            restricted = true,
        )
        val askUuid = UUID.randomUUID()
        println("Creating an ask with UUID: $askUuid, requiring all attributes: $attributes")
        createAndSendAsk(deps, askUuid, attributes, AttributeRequirementType.ALL)
        val bidUuid = UUID.randomUUID()
        println("Creating bid with UUID: $bidUuid, requiring all attributes: $attributes")
        createAndSendBid(deps, bidUuid, attributes, AttributeRequirementType.ALL)
        val executeMatch = ExecuteMatch.new(
            askId = askUuid.toString(),
            bidId = bidUuid.toString(),
        )
        println("Executing match... This should fail because the asker and bidder are both missing the attributes")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
            println("FAILURE! The contract execution should throw an exception!  Ending the example early")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to a missing asker AND bidder set of attributes. Exception message: ${e.message}")
        }
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes,
            targetAddress = deps.accounts.askerAccount.address(),
        )
        println("Executing match... This should fail because the bidder is still missing the attributes")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
            println("FAILURE! The contract execution should throw an exception! Ending the example early!")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to a missing bidder set of attributes. Exception message: ${e.message}")
        }
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes,
            targetAddress = deps.accounts.bidderAccount.address(),
        )
        println("Executing match... This should be accepted because both accounts now have all required attributes")
        client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
        client.verifyAskAndBidAreDeleted(askUuid, bidUuid)
        println("Successfully made a coin trade with required attribute type of ALL")
    }

    private fun doAnyExample(deps: AppDependencies) {
        val client = deps.client
        val attributes = deps.config.requiredAttributesConfig.attributes
        println("Binding all attribute names $attributes to admin address [${deps.accounts.adminAccount.address()}]")
        bindNamesToSigner(
            deps = deps,
            names = attributes,
            signer = deps.accounts.adminAccount,
            restricted = true,
        )
        val askUuid = UUID.randomUUID()
        println("Creating an ask with UUID: $askUuid, requiring any of attributes: $attributes")
        createAndSendAsk(deps, askUuid, attributes, AttributeRequirementType.ANY)
        val bidUuid = UUID.randomUUID()
        println("Creating bid with UUID: $bidUuid, requiring any of attributes: $attributes")
        createAndSendBid(deps, bidUuid, attributes, AttributeRequirementType.ANY)
        val executeMatch = ExecuteMatch.new(
            askId = askUuid.toString(),
            bidId = bidUuid.toString(),
        )
        println("Executing match... This should fail because the asker and bidder are both missing all the attributes")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
            println("FAILURE! The contract execution should throw an exception!  Ending the example early")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to a missing asker AND bidder set of attributes. Exception message: ${e.message}")
        }
        // Only add a random one of the attributes to the asker account to spice things up and verify that only one of
        // any of the values is required
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes.random().wrapList(),
            targetAddress = deps.accounts.askerAccount.address(),
        )
        println("Executing match... This should fail because the bidder is still missing the attributes")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
            println("FAILURE! The contract execution should throw an exception! Ending the example early!")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to a missing bidder set of attributes. Exception message: ${e.message}")
        }
        // Only add a random one of the attributes to the bidder account to spice things up and verify that only one of
        // any of the values is required
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes.random().wrapList(),
            targetAddress = deps.accounts.bidderAccount.address(),
        )
        println("Executing match... This should be accepted because both accounts now have one of the required attributes each")
        client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
        client.verifyAskAndBidAreDeleted(askUuid, bidUuid)
        println("Successfully made a coin trade with required attribute type of ANY")
    }

    private fun doNoneExample(deps: AppDependencies) {
        val client = deps.client
        val attributes = deps.config.requiredAttributesConfig.attributes
        println("Binding all attribute names $attributes to admin address [${deps.accounts.adminAccount.address()}]")
        bindNamesToSigner(
            deps = deps,
            names = attributes,
            signer = deps.accounts.adminAccount,
            restricted = true,
        )
        val firstAskUuid = UUID.randomUUID()
        println("Creating an ask with UUID: $firstAskUuid, requiring none of attributes: $attributes")
        createAndSendAsk(deps, firstAskUuid, attributes, AttributeRequirementType.NONE)
        val firstBidUuid = UUID.randomUUID()
        println("Creating bid with UUID: $firstBidUuid, requiring none of attributes: $attributes")
        createAndSendBid(deps, firstBidUuid, attributes, AttributeRequirementType.NONE)
        val firstExecuteMatch = ExecuteMatch.new(
            askId = firstAskUuid.toString(),
            bidId = firstBidUuid.toString(),
        )
        println("Executing match... This should succeeed because neither the ask nor the bid have any of the attributes")
        client.executeContract(signer = deps.accounts.adminAccount, executeMsg = firstExecuteMatch)
        client.verifyAskAndBidAreDeleted(firstAskUuid, firstBidUuid)
        val secondAskUuid = UUID.randomUUID()
        println("Creating ask with uuid: $secondAskUuid, requiring none of attributes: $attributes")
        createAndSendAsk(deps, secondAskUuid, attributes, AttributeRequirementType.NONE)
        val secondBidUuid = UUID.randomUUID()
        println("Creating bid with uuid: $secondBidUuid, requiring none of attributes: $attributes")
        createAndSendBid(deps, secondBidUuid, attributes, AttributeRequirementType.NONE)
        val secondExecuteMatch = ExecuteMatch.new(
            askId = secondAskUuid.toString(),
            bidId = secondBidUuid.toString(),
        )
        // Only add a random one of the attributes to the asker account to spice things up and verify that only one of
        // any of the values is required to cause a rejection
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes.random().wrapList(),
            targetAddress = deps.accounts.askerAccount.address(),
        )
        println("Executing match... This should fail because the asker now has one of the attributes")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = secondExecuteMatch)
            println("FAILURE! The contract execution should throw an exception! Ending the example early!")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to the asker having one of the attributes. Exception message: ${e.message}")
        }
        // Only add a random one of the attributes to the bidder account to spice things up and verify that only one of
        // any of the values is required to cause a rejection
        addDummyAttributesToAddress(
            deps = deps,
            attributeOwner = deps.accounts.adminAccount,
            attributes = attributes.random().wrapList(),
            targetAddress = deps.accounts.bidderAccount.address(),
        )
        println("Executing match... This should be rejected moreso because both accounts now have one of the required attributes each")
        try {
            client.executeContract(signer = deps.accounts.adminAccount, executeMsg = secondExecuteMatch)
            println("FAILURE! The contract execution should throw an exception! Ending the example early!")
            return
        } catch (e: Exception) {
            println("SUCCESS! The contract execution failed due to both asker and bidder having one of the attributes specified to be missing. Exception message: ${e.message}")
        }
        println("Successfully made a coin trade with required attribute type of NONE and demonstrated rejections")
    }

    private fun addDummyAttributesToAddress(
        deps: AppDependencies,
        attributeOwner: Signer,
        attributes: List<String>,
        targetAddress: String,
    ) {
        attributes.map { attributeName ->
            MsgAddAttributeRequest.newBuilder().also { addAttribute ->
                addAttribute.account = targetAddress
                addAttribute.attributeType = AttributeType.ATTRIBUTE_TYPE_STRING
                addAttribute.name = attributeName
                addAttribute.owner = attributeOwner.address()
                addAttribute.value = "dummyvalue".toByteString()
            }.build().toAny()
        }.also { attributeMsgs ->
            println("Adding attributes $attributes to address [$targetAddress] with owner address [${attributeOwner.address()}]")
            deps.client.pbClient.estimateAndBroadcastTx(
                txBody = attributeMsgs.toTxBody(),
                signers = BaseReqSigner(attributeOwner).wrapList(),
                mode = BroadcastMode.BROADCAST_MODE_BLOCK,
                gasAdjustment = 1.3,
            )
        }
    }

    private fun createAndSendAsk(
        deps: AppDependencies,
        askUuid: UUID,
        attributes: List<String>,
        requirementType: AttributeRequirementType,
    ) {
        val createAsk = CreateAsk.newCoinTrade(
            id = askUuid.toString(),
            quote = DEFAULT_QUOTE,
            descriptor = RequestDescriptor(
                description = "Example description",
                effectiveTime = OffsetDateTime.now(),
                attributeRequirement = AttributeRequirement.new(attributes, requirementType),
            )
        )
        deps.client.executeContract(
            signer = deps.accounts.askerAccount,
            executeMsg = createAsk,
            funds = DEFAULT_BASE,
        )
        deps.client.checkAsk(askUuid, createAsk.createAsk.descriptor?.effectiveTime)
    }

    private fun createAndSendBid(
        deps: AppDependencies,
        bidUuid: UUID,
        attributes: List<String>,
        requirementType: AttributeRequirementType,
    ) {
        val createBid = CreateBid.newCoinTrade(
            id = bidUuid.toString(),
            base = DEFAULT_BASE,
            descriptor = RequestDescriptor(
                description = "Example description",
                effectiveTime = OffsetDateTime.now(),
                attributeRequirement = AttributeRequirement.new(attributes, requirementType),
            )
        )
        deps.client.executeContract(
            signer = deps.accounts.bidderAccount,
            executeMsg = createBid,
            funds = DEFAULT_QUOTE,
        )
        deps.client.checkBid(bidUuid, createBid.createBid.descriptor?.effectiveTime)
    }
}
