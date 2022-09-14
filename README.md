# Toy Payments Engine

This program takes in a CSV file describing a series of unprocessed transactions, processes those transactions, and prints out the resulting state of the clients involved in those transactions (also in a CSV format).

The input CSV file should have 4 columns:

- `type` : The action of a transaction
    - Recorded types (we need to record these since they can be disputed)
        - `deposit` : Adds funds to a client's account (available+, total+)
        - `withdrawal` : Removes funds from a client's account (available-, total-)
    - Non-Recorded types (no need to record these since they only reference other transactions)
        - `dispute` : Holds the funds of the referenced transaction (available-, held+)
        - `resolve` : Releases the funds of a disputed transaction (available+, held-)
        - `chargeback` : Withdraws held funds of a disputed transaction (held-, total-, locked)
- `client` : The unique `u16` identifier of a client
- `tx` : The unique `u32` identifier of a transaction
- `amount` : The amount of funds for a transaction

The output should also be in CSV form with 5 columns:

- `client` : The unique `u16` identifier of a client
- `available` : The amount of available funds (to 4 decimal places)
- `held` : The amount of held funds
- `total` : The total amount of funds in the client's account
- `locked` : Whether the account is locked from a chargeback

> _Note: all float values are precise to 4 decimal places._

## Getting Started

To run this program you need to generate a CSV file to process. You can do this manually or use the included Python 3 script like this for example:

```
$ python3 gentx.py 100 transactions.csv
```

This creates a `transactions.csv` files with 100 random rows.

Finally, use that CSV as input to the program like this:

```
$ cargo run -- transactions.csv > accounts.csv
```

There are several tests you can run as well using `cargo test`.

## Handling Disputes

The general "algorithm" for processing disputes goes like this:

```
dispute    : available -= amount | held += amount | total
resolve    : available += amount | held -= amount | total
chargeback : available           | held -= amount | total -= amount
```

> NOTE: `amount` can be either positive (for a deposit) or negative (for a withdrawal).

### Disputing a Deposit

In this case a credit (+amount) is being disputed so the amount of that transaction is subtracted from the available funds and added to the held funds.

If this is resolved, we undo this and we go back to the state where the funds were indeed deposited.

Example:

|         Tx | amount | avail. | held | total | locked |
|-----------:|-------:|-------:|-----:|------:|--------|
| deposit    |    5.0 |    5.0 |  0.0 |   5.0 |  false |
| dispute    |    5.0 |    0.0 |  5.0 |   5.0 |  false |
| resolve    |    5.0 |    5.0 |  0.0 |   5.0 |  false |

If this is chargebacked, we commit to this and remove those funds and lock the account.

Example:

|         Tx | amount | avail. | held | total | locked |
|-----------:|-------:|-------:|-----:|------:|--------|
| deposit    |    5.0 |    5.0 |  0.0 |   5.0 |  false |
| dispute    |    5.0 |    0.0 |  5.0 |   5.0 |  false |
| chargeback |    5.0 |    0.0 |  0.0 |   0.0 |   true |

### Disputed Withdrawal

In this case a debit (-amount) is being disputed so the amount of that transaction is added to the available funds and subtracted from the held funds.

If this is resolved, we undo this and we go back to the state where the funds were indeed withdrawn.

Example:

|         Tx | amount | avail. | held | total | locked |
|-----------:|-------:|-------:|-----:|------:|--------|
| deposit    |    5.0 |    5.0 |  0.0 |   5.0 |  false |
| withdrawal |   -2.0 |    3.0 |  0.0 |   3.0 |  false |
| dispute    |   -2.0 |    5.0 | -2.0 |   3.0 |  false |
| resolve    |   -2.0 |    3.0 |  0.0 |   3.0 |  false |

If this is chargebacked, we commit to this and add those funds back and lock the account.

Example:

|         Tx | amount | avail. | held | total | locked |
|-----------:|-------:|-------:|-----:|------:|--------|
| deposit    |    5.0 |    5.0 |  0.0 |   5.0 |  false |
| withdrawal |   -2.0 |    3.0 |  0.0 |   3.0 |  false |
| dispute    |   -2.0 |    5.0 | -2.0 |   3.0 |  false |
| chargeback |   -2.0 |    5.0 |  0.0 |   5.0 |   true |

## Assumptions

I'm making several assumptions in order to simplify things a bit. These scenarios will be ignored and treated as errors in the input CSV.

### Re-Disputing

If a disputed transaction that _has not been resolved_ (i.e. it's currently in either the `disputed` or `chargebacked` state) it may not be re-disputed.

### Locked Accounts

Locked accounts are locked forever. Obviously this is unrealistic but due to the limited set of transactions in the input and the limited amount of time I have to work on this, I'm going with simplicity here.

If an account has been locked (can only be due to a `chargeback`) then deposits, withdraws, or more disputes are disallowed. Again, this is for simplicity.
