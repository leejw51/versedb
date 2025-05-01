#!/usr/bin/env python3

import argparse
import asyncio
import capnp
import versedb_capnp


async def main():
    parser = argparse.ArgumentParser(
        usage="Connects to the Versedb server at the given address/port"
    )
    parser.add_argument(
        "--host", default="localhost", help="Host address (default: localhost)"
    )
    parser.add_argument(
        "--port", type=int, default=8000, help="Port number (default: 8000)"
    )
    args = parser.parse_args()

    print(f"🔌 Connecting to server at {args.host}:{args.port}...")

    stream = await capnp.AsyncIoStream.create_connection(args.host, args.port)
    client = capnp.TwoPartyClient(stream)
    versedb = client.bootstrap().cast_as(versedb_capnp.Versedb)

    # Test helloworld
    print("\nTesting helloworld...")
    result = await versedb.helloworld("World")
    print(f"Server says: {result.output}")

    # Test add
    print("\nTesting add...")
    key = b"test_key"
    value = b"test_value"
    await versedb.add(key, value)
    print(f"Added key-value pair: {key} -> {value}")

    # Test select
    print("\nTesting select...")
    result = await versedb.select(key)
    print(f"Selected value: {result.value}")

    # Test selectRange
    print("\nTesting selectRange...")
    range = versedb_capnp.KeyRange.new_message()
    range.start = b"test"
    range.end = b"test_z"
    result = await versedb.selectRange(range)
    print("Selected range results:")
    for pair in result.pairs:
        print(f"  {pair.key} -> {pair.value}")

    # Test removeRange
    print("\nTesting removeRange...")
    range = versedb_capnp.KeyRange.new_message()
    range.start = b"test"
    range.end = b"test_z"
    result = await versedb.removeRange(range)
    print("Removed range results:")
    for pair in result.pairs:
        print(f"  {pair.key} -> {pair.value}")

    # Test remove
    print("\nTesting remove...")
    await versedb.remove(key)
    print(f"Removed key: {key}")

    # Verify removal
    print("\nVerifying removal...")
    result = await versedb.select(key)
    print(f"Value after removal: {result.value}")


if __name__ == "__main__":
    asyncio.run(capnp.run(main()))
