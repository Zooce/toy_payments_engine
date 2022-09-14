import csv
import random
import sys

CLIENT_COUNT = random.randrange(1, 1000)
NEXT_TX_ID = 0


def get_float():
    return random.uniform(-1_000_000, 1_000_000)


def get_rand_row():
    global NEXT_TX_ID
    TXS = ['deposit', 'withdrawal', 'dispute', 'resolve', 'chargeback']
    WTS = [35, 35, 10, 10, 10]
    tx = random.choices(TXS, weights=WTS)[0]
    client = random.randrange(1, CLIENT_COUNT, 1)
    if tx in ['deposit', 'withdrawal']:
        NEXT_TX_ID += 1
        return [tx, client, NEXT_TX_ID, f'{get_float():.4f}']
    else:
        return [tx, client, random.randrange(1, NEXT_TX_ID + 2, 1), '']  # we may get a transaction ID that doesn't exist yet - we wan't this


def write_header(writer):
    writer.writerow(['type', 'client', 'tx', 'amount'])


def main(rowcount, output):
    rows = [get_rand_row() for _ in range(rowcount)]
    with open(output, 'w', newline='') as output:
        writer = csv.writer(output)
        write_header(writer)
        writer.writerows(rows)


if __name__ == "__main__":
    rowcount = int(sys.argv[1])
    output = sys.argv[2]
    main(rowcount, output)
