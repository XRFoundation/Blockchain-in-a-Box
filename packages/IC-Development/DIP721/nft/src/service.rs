use crate::management::is_fleek;
use crate::management::Fleek;
use crate::types::*;
use crate::utils::*;

use std::str::FromStr;

use ic_kit::Principal;
use ic_kit::ic;
use ic_kit::ic::caller;
use ic_kit::ic::trap;
use ic_kit::macros::*;
use ic_kit::ic::call;
use ic_kit::ic::call_raw;
use ic_kit::RejectionCode;

use ic_cdk::api::print;

use cap_sdk::handshake;
use cap_sdk::DetailValue;
use cap_sdk::IndefiniteEventBuilder;

pub use ic_kit::candid::{candid_method, CandidType, Deserialize, Int, Nat};

/// HEALTH-CHECK ///
#[query]
fn name() -> String {
    String::from("NFT Canister")
}

/// BEGIN DIP-721 ///
#[query(name = "balanceOfDip721")]
fn balance_of_dip721(user: Principal) -> u64 {
    ledger().balance_of(&user.into())
}

#[query(name = "ownerOfDip721")]
fn owner_of_dip721(token_id: u64) -> Result<Principal, ApiError> {
    ledger().owner_of(&token_id.to_string())
}

#[update(name = "safeTransferFromDip721")]
async fn safe_transfer_from_dip721(_from: Principal, to: Principal, token_id: u64) -> TxReceipt {
    if !is_fleek(&ic::caller()) {
        return Err(ApiError::Unauthorized);
    }
    assert_ne!(
        to,
        Principal::from_slice(&[0; 29]),
        "transfer request to cannot be the zero principal"
    );

    ledger().transfer(
        &User::principal(caller()),
        &User::principal(to),
        &token_id.to_string(),
    );

    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("transfer")
        .details(vec![
            ("from".into(), DetailValue::Principal(caller())),
            ("to".into(), DetailValue::Principal(to)),
            ("token_id".into(), DetailValue::U64(token_id)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();

    Ok(tx_id.into())
}

#[update(name = "transferFromDip721")]
async fn transfer_from_dip721(_from: Principal, to: Principal, token_id: u64) -> TxReceipt {
    if !is_fleek(&ic::caller()) {
        return Err(ApiError::Unauthorized);
    }
    assert_ne!(
        caller(),
        to,
        "transfer request caller and to cannot be the same"
    );

    ledger().transfer(
        &User::principal(caller()),
        &User::principal(to),
        &token_id.to_string(),
    );

    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("transfer")
        .details(vec![
            ("from".into(), DetailValue::Principal(caller())),
            ("to".into(), DetailValue::Principal(to)),
            ("token_id".into(), DetailValue::U64(token_id)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();

    Ok(tx_id)
}

#[query(name = "supportedInterfacesDip721")]
fn supported_interfaces_dip721() -> Vec<InterfaceId> {
    vec![InterfaceId::Mint, InterfaceId::TransactionHistory]
}

#[query(name = "logoDip721")]
fn logo_dip721() -> LogoResult {
    unimplemented!();
}

#[query(name = "nameDip721")]
fn name_dip721() -> &'static str {
    &token_level_metadata().name
}

#[query(name = "symbolDip721")]
fn symbol_dip721() -> &'static str {
    &token_level_metadata().symbol
}

#[query(name = "totalSupplyDip721")]
fn total_supply_dip721() -> u64 {
    ledger().total_supply()
}

#[query(name = "getMetadataDip721")]
fn get_metadata_dip721(token_id: u64) -> MetadataResult {
    ledger().get_metadata(token_id)
}

#[query(name = "getMaxLimitDip721")]
fn get_max_limit_dip721() -> u16 {
    200
}

#[allow(unreachable_code, unused_variables)]
#[query(name = "getMetadataForUserDip721")]
fn get_metadata_for_user_dip721(user: Principal) -> Vec<ExtendedMetadataResult> {
    ledger().get_metadata_for_user(&user)
}

#[allow(unreachable_code, unused_variables)]
#[query(name = "getTokenIdsForUserDip721")]
fn get_token_ids_for_user_dip721(user: Principal) -> Vec<u64> {
    ledger().get_token_ids_for_user(&user)
}

#[update(name = "mintDip721")]
async fn mint_dip721(to: Principal, metadata_desc: MetadataDesc) -> MintReceipt {
    if !is_fleek(&ic::caller()) {
        return Err(ApiError::Unauthorized);
    }

    let response = ledger().mintNFT(&to, &metadata_desc).unwrap();
    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("mint")
        .details(vec![
            ("to".into(), DetailValue::Principal(to)),
            ("token_id".into(), DetailValue::U64(response.token_id)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();

    Ok(MintReceiptPart {
        token_id: response.token_id,
        id: tx_id.into(),
    })
}

/// END DIP-721 ///

#[update(name = "setWicpId")]
async fn set_wicp_canister(canister_id: Principal) {
    if is_fleek(&ic::caller()) {
        token_level_metadata().payment = Some(canister_id);
    }
}

#[query(name = "getWicpId")]
async fn get_wicp() -> Principal {
    wicp_canister_id()
}


#[update(name = "listForSale")]
async fn list_for_sale(token_id: u64, price: Nat) -> Result <bool, String> {
    ledger().list(ic_kit::ic::caller(), &token_id.to_string(), price).await
}

#[update(name = "delistFromSale")]
async fn delist_from_sale(token_id: u64) -> Result <bool, String> {
    ledger().delist(ic_kit::ic::caller(), &token_id.to_string()).await
}

#[update(name = "buyDip721")]
async fn buy_dip721(token_id: u64) -> Result <WicpTxReceipt, String> {

    let listingResult: Option<Listing> = ledger().get_listing(&token_id.to_string()).await;
    
    if listingResult.is_none() {
        return Err("Token not listed for sale / Token with such ID does not exist".to_string());
    } else {
        let listing: Listing = listingResult.unwrap().clone();

        let method_caller: Principal = ic::caller();
        
        let response: Result<(WicpTxReceipt,), (RejectionCode, String)> = call(
                            wicp_canister_id(),
                            "transferFrom",
                            (method_caller, listing.owner, listing.price.clone()),
        ).await;
        
        if response.is_ok() {

            match response.ok() {

                // Unwrapping WicpTxReciept which is a Result<Nat, WicpTxError>
                // Only Nat response means success
                Some(wicpReceiptTuple) => {
                    // All OK, transfer ownership and delist token
                    let wicpReceipt: WicpTxReceipt = wicpReceiptTuple.0;
                    if (wicpReceipt.is_ok()) {
                        let delisted = ledger().delist(listing.owner, &token_id.to_string()).await;

                        if delisted.is_ok() {
                            ledger().transfer(&User::principal(listing.owner), &User::principal(method_caller), &token_id.to_string());
                            return Ok(wicpReceipt);
                        } else {
                            return Err("Er-1: Unexpected error occured".to_string());
                        }
                    } else {
                        return Ok(wicpReceipt);
                        // In this branch we return the WicpError which can be any of:
                        /*
                            InsufficientBalance,
                            InsufficientAllowance,
                            Unauthorized,
                            LedgerTrap,
                            AmountTooSmall,
                            BlockUsed,
                            ErrorOperationStyle,
                            ErrorTo,
                            Other,
                        */
                    }
                },
                None => {
                    // Ideally funds should be returned at this point
                    // I doubt code will ever reach this branch of logic though
                    return Err("Er-3: Unexpected error occured".to_string());
                }
            }
            
        } else {
            // print(&response.err().unwrap().1);
            return Err("Unexpected error occured while calling external canister".to_string());
        }
    }
}


#[update]
async fn transfer(transfer_request: TransferRequest) -> TransferResponse {
    if !is_fleek(&ic::caller()) {
        return Err(TransferError::Unauthorized("Not Admin".to_string()));
    }
    expect_principal(&transfer_request.from);
    expect_principal(&transfer_request.to);
    assert_ne!(
        transfer_request.from, transfer_request.to,
        "transfer request from and to cannot be the same"
    );
    assert_eq!(transfer_request.amount, 1, "only amount 1 is supported");
    expect_caller_general(&transfer_request.from, transfer_request.subaccount);

    ledger().transfer(
        &User::principal(caller()),
        &transfer_request.to,
        &transfer_request.token,
    );

    let token_id = &transfer_request.token.parse::<u64>().unwrap();

    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("transfer")
        .details(vec![
            (
                "from".into(),
                user_to_detail_value(User::principal(caller())),
            ),
            ("to".into(), user_to_detail_value(transfer_request.to)),
            ("token_id".into(), DetailValue::U64(*token_id)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();

    Ok(Nat::from(tx_id))
}

#[allow(non_snake_case, unreachable_code, unused_variables)]
#[update]
async fn mintNFT(mint_request: MintRequest) -> Option<TokenIdentifier> {
    trap("Disabled as current EXT metadata doesn't allow multiple assets per token");
    if !is_fleek(&ic::caller()) {
        return None;
    }
    expect_principal(&mint_request.to);
    expect_caller(&token_level_metadata().owner.expect("token owner not set"));

    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("mint")
        .details(vec![
            ("to".into(), user_to_detail_value(mint_request.to)),
            ("token_id".into(), DetailValue::U64(123)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();
    Some(tx_id.to_string())
}

#[query]
fn bearer(token_identifier: TokenIdentifier) -> AccountIdentifierReturn {
    ledger().bearer(&token_identifier)
}

#[allow(unreachable_code, unused_variables)]
#[query(name = "getAllMetadataForUser")]
fn get_all_metadata_for_user(user: User) -> Vec<TokenMetadata> {
    trap("Disabled as current EXT metadata doesn't allow multiple assets per token");
    ledger().get_all_metadata_for_user(&user)
}

#[query]
fn supply(token_identifier: TokenIdentifier) -> BalanceReturn {
    ledger().supply(&token_identifier)
}

#[allow(unreachable_code, unused_variables)]
#[query]
fn metadata(token_identifier: TokenIdentifier) -> MetadataReturn {
    trap("Disabled as current EXT metadata doesn't allow multiple assets per token");
    ledger().metadata(&token_identifier)
}

#[update]
async fn add(transfer_request: TransferRequest) -> Option<TransactionId> {
    if !is_fleek(&ic::caller()) {
        return None;
    }
    expect_principal(&transfer_request.from);
    expect_principal(&transfer_request.to);

    let token_id = &transfer_request.token.parse::<u64>().unwrap();

    let event = IndefiniteEventBuilder::new()
        .caller(caller())
        .operation("transfer_from")
        .details(vec![
            ("to".into(), user_to_detail_value(transfer_request.to)),
            ("from".into(), user_to_detail_value(transfer_request.from)),
            ("token_id".into(), DetailValue::U64(*token_id)),
        ])
        .build()
        .unwrap();

    let tx_id = insert_into_cap(event).await.unwrap();

    Some(Nat::from(tx_id))
}

fn store_data_in_stable_store() {
    let data = StableStorageBorrowed {
        ledger: ledger(),
        token: token_level_metadata(),
        fleek: fleek_db(),
    };
    ic::stable_store((data,)).expect("failed");
}

fn restore_data_from_stable_store() {
    let (data,): (StableStorage,) = ic::stable_restore().expect("failed");
    ic::store(data.ledger);
    ic::store(data.token);
    ic::store(data.fleek);
}

#[init]
fn init(owner: Principal, symbol: String, name: String, history: Principal, payment: Principal) {
    ic::store(Fleek(vec![owner, ic::caller()]));
    *token_level_metadata() = TokenLevelMetadata::new(Some(owner), symbol, name, Some(history), Some(payment)); // Payment is ID of WICP for now
    handshake(1_000_000_000_000, Some(history));
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::api::print(format!("Executing preupgrade"));
    store_data_in_stable_store();
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::api::print(format!("Executing postupgrade"));
    restore_data_from_stable_store();
}
