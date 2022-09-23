.. _create-tx-details:

GDK Create Transaction JSON
===========================

This section details how to create various kinds of transaction using
`GA_create_transaction`. Once created, the resulting JSON is generally passed
to `GA_sign_transaction` to obtain signatures, then broadcast to the network
via `GA_send_transaction` or `GA_broadcast_transaction`.

Overview
--------

The caller passes details about the transaction they would like to construct.
The returned JSON contains the resulting transaction and an ``"error"`` element
which is either empty if the call suceeded in creating a valid transaction or
contains an error code describing the problem.

Building transactions can be done iteratively, by passing the result of one
call into the next after making changes to the returned JSON. This is useful for
manual transaction creation as it allows users to see the effect of
changes such as different fee rates interactively, and to fix errors on the fly.

When using gdk as a integration solution, `GA_create_transaction` is generally
only called once, and if an error occurs the operation is aborted.

Note that the returned JSON will contain additional elements beyond those
documented here. The caller should not attempt to change these elements; the
documented inputs are the only user-level changes that should be made, and
the internal elements may change name or meaning from release to release.

Mandatory and Optional Elements
-------------------------------

Only two elements are always mandatory: ``"addressees"`` and ``"utxos"``. A
transaction sending some amount from the wallet can be created using e.g:

.. code-block:: json

  {
    "addressees": [ {} ],
    "utxos": { }
  }


:addressees: Mandatory. An array of :ref:`addressee` elements, one for each recipient.
:utxos: Mandatory. The UTXOs to fund the transaction with, :ref:`unspent-outputs` as
        returned by `GA_get_unspent_outputs`. Any UTXOs present are candidates for
        inclusion in the transaction.

Optional elements allow more precise control over the transaction:

:fee_rate: Defaults to the sessions default fee rate setting. The fee rate in
           satoshi per 1000 bytes to use for fee calculation.
:utxo_strategy: Defaults to ``"default"``. Set to ``"manual"`` for manual UTXO
                selection.
:send_all: Defaults to ``false``. If set to ``true``, all given UTXOs will be
           sent and no change output will be created.
:is_partial: Defaults to ``false``. Used for creating partial/incomplete
             transactions such as half-swaps. If set to ``true``, no change
             output will be created and fees will not be calculated or deducted
             from inputs. Consider using `GA_create_swap_transaction` instead
             of manually setting this element.
:randomize_inputs: Defaults to ``true``. If set to ``true``, the
                   order of the used UTXOs in the created transaction is randomized.

.. _addressee:

Addressee JSON
--------------

Describes an intended recipient for a transaction.

.. code-block:: json

  {
    "address": "2NFHMw7GbqnQ3kTYMrA7MnHiYDyLy4EQH6b",
    "satoshi": 100000,
    "asset_id": "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d"
  }

:address: Mandatory. The address to send to. All address types for the network are supported.
          Additionally, `BIP 21 <https://github.com/bitcoin/bips/blob/master/bip-0021.mediawiki>`_
          URLs are supported along with the `Liquid adaptation <https://github.com/ElementsProject/elements/issues/805>`_.
          Note that BIP 70 payment requests are not supported.
:satoshi: Mandatory. The amount to send to the recipient in satoshi.
:asset_id: Mandatory for Liquid, must not be present for Bitcoin. The asset to be
           sent to the recipient, in display hex format.

Sweeping
--------

A sweep transaction moves coins from an address with a known private key to
another address. Unlike a simple send transaction, the coins to be moved are
not associated with the users wallet in any way. Sweeping is typically used
to move coins from a paper wallet into the users wallet.

To create a sweep transaction, pass the following json to `GA_create_transaction`:

.. code-block:: json

  {
    "addressees": [ {} ],
    "private_key": "mrWqGcXTrZpqQvvLwN63amstf8no1W8oo6"
  }

:addressees: Mandatory. Pass a single :ref:`addressee` element for the coin destination.
:private_key: Mandatory. The private key for the coin to sweep, in either
                          `BIP 38 <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>`_
                          or `Wallet Import Format <https://en.bitcoin.it/wiki/Wallet_import_format>`_.

Note that ``"send_all"`` will always be automatically set to ``true`` for sweep transactions.
