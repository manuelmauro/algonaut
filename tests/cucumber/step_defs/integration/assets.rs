use crate::step_defs::integration::world::World;
use algonaut::error::Error;
use algonaut_core::{Address, AssetId, TransactionId};
use algonaut_model::algod::AssetParams;
use algonaut_transaction::{
    AcceptAsset, ClawbackAsset, CreateAsset, FreezeAsset, TransferAsset,
    builder::{DestroyAsset, UpdateAsset},
};
use cucumber::{given, then, when};

const UNIT_NAME: &str = "unit";
const ASSET_NAME: &str = "name";
const ASSET_URL: &str = "http://someurl";
const METADATA_HASH: &[u8] = b"fACPO4nRgO55j1ndAK3W6Sgc4APkc";

fn pad_metadata_hash() -> Vec<u8> {
    let mut buf = METADATA_HASH.to_vec();
    buf.resize(32, 0);
    buf
}

/// Return (creator, second_account) — two distinct funded wallet
/// accounts. kmd's list_keys ordering is not deterministic, and the
/// sandbox preloads several accounts with balance; we pick the two
/// with the highest balances so the asset flow has both a creator and
/// a recipient.
async fn choose_creator_and_second(w: &World) -> Result<(Address, Address), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let accounts = w.accounts.as_ref().expect("accounts not set");
    let mut balances: Vec<(Address, u64)> = Vec::with_capacity(accounts.len());
    for addr in accounts {
        let info = algod.account(addr).await?;
        balances.push((*addr, info.amount));
    }
    balances.sort_by(|a, b| b.1.cmp(&a.1));
    if balances.len() < 2 || balances[1].1 == 0 {
        return Err(Error::Msg(
            "could not find two funded accounts in the wallet".to_string(),
        ));
    }
    Ok((balances[0].0, balances[1].0))
}

#[given(expr = "asset test fixture")]
async fn asset_test_fixture(w: &mut World) {
    w.expected_asset_params = None;
    w.asset_id = None;
}

async fn build_creation(w: &mut World, total: u64, default_frozen: bool) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let (creator, second) = choose_creator_and_second(w).await?;
    w.asset_creator = Some(creator);
    w.asset_second = Some(second);

    let params = algod.suggested_params().await?;
    let tx = CreateAsset::new(creator, total, 0, default_frozen)
        .unit_name(UNIT_NAME.to_string())
        .asset_name(ASSET_NAME.to_string())
        .url(ASSET_URL.to_string())
        .meta_data_hash(pad_metadata_hash())
        .manager(creator)
        .reserve(creator)
        .freeze(creator)
        .clawback(creator)
        .build(&params)?;

    w.tx = Some(tx);

    w.expected_asset_params = Some(AssetParams {
        creator: creator.to_string(),
        total,
        decimals: 0,
        default_frozen: Some(default_frozen),
        unit_name: Some(UNIT_NAME.to_string()),
        name: Some(ASSET_NAME.to_string()),
        url: Some(ASSET_URL.to_string()),
        manager: Some(creator.to_string()),
        reserve: Some(creator.to_string()),
        freeze: Some(creator.to_string()),
        clawback: Some(creator.to_string()),
        metadata_hash: None,
        name_b64: None,
        unit_name_b64: None,
        url_b64: None,
    });
    Ok(())
}

#[given(regex = r"^default asset creation transaction with total issuance (\d+)$")]
async fn default_asset_creation_transaction(w: &mut World, total: u64) -> Result<(), Error> {
    build_creation(w, total, false).await
}

#[given(regex = r"^default-frozen asset creation transaction with total issuance (\d+)$")]
async fn default_frozen_asset_creation_transaction(w: &mut World, total: u64) -> Result<(), Error> {
    build_creation(w, total, true).await
}

#[when(expr = "I send the kmd-signed transaction")]
#[then(expr = "I send the kmd-signed transaction")]
async fn i_send_the_kmd_signed_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let bytes = w
        .kmd_signed_tx_bytes
        .as_ref()
        .expect("kmd-signed transaction not set");
    match algod.send_raw(bytes).await {
        Ok(resp) => {
            w.transaction_id = Some(TransactionId(resp.tx_id));
            w.last_send_succeeded = Some(true);
        }
        Err(e) => {
            // Surface the error: the calling step `I wait for the
            // transaction to be confirmed.` will fail downstream
            // anyway, but the original error is what diagnoses the
            // root cause.
            return Err(Error::Msg(format!("send_raw (kmd-signed) failed: {e:?}")));
        }
    }
    Ok(())
}

#[when(expr = "I send the bogus kmd-signed transaction")]
async fn i_send_the_bogus_kmd_signed_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let bytes = w
        .kmd_signed_tx_bytes
        .as_ref()
        .expect("kmd-signed transaction not set");
    w.last_send_succeeded = Some(algod.send_raw(bytes).await.is_ok());
    Ok(())
}

#[when(expr = "I update the asset index")]
async fn i_update_the_asset_index(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let transaction_id = w.transaction_id.as_ref().expect("no last tx id");
    let pending = algod.pending_transaction(transaction_id).await?;
    if let Some(asset_index) = pending.asset_index {
        w.asset_id = Some(AssetId(asset_index));
    }
    Ok(())
}

#[when(expr = "I get the asset info")]
#[then(expr = "I get the asset info")]
async fn i_get_the_asset_info(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let asset = algod.asset(asset_id).await?;
    w.asset_info = Some(*asset.params);
    Ok(())
}

#[then(expr = "the asset info should match the expected asset info")]
async fn the_asset_info_should_match_the_expected_asset_info(w: &mut World) {
    let expected = w
        .expected_asset_params
        .as_ref()
        .expect("expected asset params not set");
    let actual = w.asset_info.as_ref().expect("asset info not fetched");
    assert_eq!(expected.creator, actual.creator);
    assert_eq!(expected.total, actual.total);
    assert_eq!(expected.decimals, actual.decimals);
    assert_eq!(
        expected.default_frozen.unwrap_or(false),
        actual.default_frozen.unwrap_or(false)
    );
    assert_eq!(expected.unit_name, actual.unit_name);
    assert_eq!(expected.name, actual.name);
    assert_eq!(expected.url, actual.url);
    assert_eq!(expected.manager, actual.manager);
    assert_eq!(expected.reserve, actual.reserve);
    assert_eq!(expected.freeze, actual.freeze);
    assert_eq!(expected.clawback, actual.clawback);
}

#[when(expr = "I create a no-managers asset reconfigure transaction")]
async fn i_create_a_no_managers_asset_reconfigure_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(UpdateAsset::new(creator, asset_id).build(&params)?);
    if let Some(exp) = w.expected_asset_params.as_mut() {
        exp.manager = None;
        exp.reserve = None;
        exp.freeze = None;
        exp.clawback = None;
    }
    Ok(())
}

#[when(expr = "I create an asset destroy transaction")]
async fn i_create_an_asset_destroy_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(DestroyAsset::new(creator, asset_id).build(&params)?);
    Ok(())
}

#[then(expr = "I should be unable to get the asset info")]
async fn i_should_be_unable_to_get_the_asset_info(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    let asset_id = w.asset_id.expect("asset id not set");
    assert!(
        algod.asset(asset_id).await.is_err(),
        "expected asset info to be missing after destroy"
    );
}

#[when(expr = "I create a transaction for a second account, signalling asset acceptance")]
async fn i_create_a_transaction_for_a_second_account_signalling_asset_acceptance(
    w: &mut World,
) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(AcceptAsset::new(second, asset_id).build(&params)?);
    Ok(())
}

#[when(
    regex = r"^I create a transaction transferring (\d+) assets from creator to a second account$"
)]
async fn i_create_transfer_creator_to_second(w: &mut World, amount: u64) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(TransferAsset::new(creator, asset_id, amount, second).build(&params)?);
    Ok(())
}

#[when(
    regex = r"^I create a transaction transferring (\d+) assets from a second account to creator$"
)]
async fn i_create_transfer_second_to_creator(w: &mut World, amount: u64) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(TransferAsset::new(second, asset_id, amount, creator).build(&params)?);
    Ok(())
}

#[when(regex = r"^I create a transaction revoking (\d+) assets from a second account to creator$")]
async fn i_create_revocation(w: &mut World, amount: u64) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(ClawbackAsset::new(creator, asset_id, amount, second, creator).build(&params)?);
    Ok(())
}

#[when(expr = "I create a freeze transaction targeting the second account")]
async fn i_create_a_freeze_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(FreezeAsset::new(creator, second, asset_id, true).build(&params)?);
    Ok(())
}

#[when(expr = "I create an un-freeze transaction targeting the second account")]
async fn i_create_an_unfreeze_transaction(w: &mut World) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let second = w.asset_second.expect("asset second account not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let params = algod.suggested_params().await?;
    w.tx = Some(FreezeAsset::new(creator, second, asset_id, false).build(&params)?);
    Ok(())
}

#[then(regex = r"^the creator should have (\d+) assets remaining$")]
async fn the_creator_should_have_assets_remaining(
    w: &mut World,
    expected: u64,
) -> Result<(), Error> {
    let algod = w.algod.as_ref().expect("algod not set");
    let creator = w.asset_creator.expect("asset creator not set");
    let asset_id = w.asset_id.expect("asset id not set");
    let info = algod.account(&creator).await?;
    let holding = info
        .assets
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .find(|h| h.asset_id == asset_id.0)
        .ok_or_else(|| Error::Msg(format!("creator does not hold asset {asset_id}")))?;
    assert_eq!(
        holding.amount, expected,
        "creator holds {} of asset {asset_id}, expected {expected}",
        holding.amount
    );
    Ok(())
}
