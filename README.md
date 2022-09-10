# Toy Payments Engine

This program takes in a CSV file describing a series of unprocessed transactions, processes those transactions, and prints out the resulting state of the clients involved in those transactions (also in a CSV format).

The input CSV file should have 4 columns:

- `type` : The action of a transaction
    - `deposit` : Adds funds to a client's account (available+, total+)
    - `withdraw` : Removes funds from a client's account (available-, total-)
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

## Handling Disputes

0. dispute -> [put funds on hold (no longer available)]
1. resolve -> [release funds from dispute hold (available again)]
2. chargeback -> [withdraw the held disputed funds]
