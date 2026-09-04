use std::str::FromStr;

use bdk_wallet::SignOptions;
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, OutPoint, Txid};

use crate::error::WalletError;
use crate::node::Node;
use crate::wallet::Wallet;

/// A spend request assembled from CLI arguments.
pub struct SpendRequest {
    /// Destination address, already checked against the wallet network.
    pub recipient: Address,
    /// Amount to send. Ignored when `drain` is set.
    pub amount: Amount,
    /// Optional explicit fee rate; when `None` the node's estimate is used.
    pub fee_rate: Option<FeeRate>,
    /// If non empty, only these UTXOs may be spent (manual coin selection).
    pub selected_utxos: Vec<OutPoint>,
    /// Send the whole spendable balance to `recipient` instead of `amount`.
    pub drain: bool,
}

impl SpendRequest {
    pub fn parse(
        to: &str,
        amount_sat: u64,
        fee_rate: Option<u64>,
        utxos: &[String],
        drain: bool,
        network: bdk_wallet::bitcoin::Network,
    ) -> Result<Self, WalletError> {
        let recipient = Address::from_str(to)
            .map_err(|e| WalletError::Address(e.to_string()))?
            .require_network(network)
            .map_err(|_| WalletError::Address(format!("address {to} is not valid on {network}")))?;

        let fee_rate = match fee_rate {
            Some(rate) => Some(FeeRate::from_sat_per_vb(rate).ok_or_else(|| {
                WalletError::BuildTx(format!("fee rate {rate} sat/vB overflows"))
            })?),
            None => None,
        };

        let selected_utxos = utxos
            .iter()
            .map(|raw| {
                OutPoint::from_str(raw)
                    .map_err(|e| WalletError::BuildTx(format!("bad --utxo '{raw}': {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(SpendRequest {
            recipient,
            amount: Amount::from_sat(amount_sat),
            fee_rate,
            selected_utxos,
            drain,
        });
    }
}

/// Outcome of a successful send.
pub struct SpendOutcome {
    pub txid: Txid,
    pub fee: Amount,
    pub sent: Amount,
    /// The raw transaction as the node now reports it, proving the broadcast.
    pub raw_hex: String,
}

/// Build, sign and broadcast a payment.
///
/// BDK does the heavy lifting here: `build_tx` runs branch-and-bound coin
/// selection (unless we pin the inputs), adds a change output on the internal
/// keychain, and returns a PSBT. `wallet.sign` fills in the witness, and only
/// then does the transaction leave this process, via `bitcoincore-rpc`.
pub fn send(
    wallet: &mut Wallet,
    node: &Node,
    request: SpendRequest,
) -> Result<SpendOutcome, WalletError> {
    let balance = wallet.balance();
    let spendable = balance.trusted_spendable();
    if !request.drain && request.amount > spendable {
        return Err(WalletError::InsufficientFunds {
            need: request.amount.to_sat(),
            available: spendable.to_sat(),
        });
    }

    let fee_rate = request.fee_rate.unwrap_or_else(|| node.fee_rate());
    let recipient_script = request.recipient.script_pubkey();

    let mut psbt = {
        let mut builder = wallet.inner_mut().build_tx();
        builder.fee_rate(fee_rate);

        if request.drain {
            builder.drain_wallet();
            builder.drain_to(recipient_script.clone());
        } else {
            builder.add_recipient(recipient_script.clone(), request.amount);
        }

        if !request.selected_utxos.is_empty() {
            builder.manually_selected_only();
            builder
                .add_utxos(&request.selected_utxos)
                .map_err(|e| WalletError::BuildTx(e.to_string()))?;
        }

        builder
            .finish()
            .map_err(|e| WalletError::BuildTx(e.to_string()))?
    };

    let finalized = wallet
        .inner()
        .sign(&mut psbt, SignOptions::default())
        .map_err(|e| WalletError::Sign(e.to_string()))?;
    if !finalized {
        return Err(WalletError::Sign(
            "PSBT could not be finalized with the wallet's keys".to_string(),
        ));
    }

    let tx = psbt
        .extract_tx()
        .map_err(|e| WalletError::Sign(e.to_string()))?;

    let fee = wallet
        .inner()
        .calculate_fee(&tx)
        .map_err(|e| WalletError::BuildTx(e.to_string()))?;

    // Persist the revealed change index before broadcasting so a crash right
    // after the send cannot make us reuse that change address.
    wallet.persist()?;

    let txid = node.broadcast(&tx)?;
    let raw_hex = node.raw_transaction_hex(&txid)?;

    let sent = tx
        .output
        .iter()
        .find(|out| out.script_pubkey == recipient_script)
        .map(|out| out.value)
        .unwrap_or(request.amount);

    return Ok(SpendOutcome {
        txid,
        fee,
        sent,
        raw_hex,
    });
}
