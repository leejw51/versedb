#!/usr/bin/env python3

import argparse
import asyncio
import capnp
import versedb_capnp
from typing import Dict


class VersedbImpl(versedb_capnp.Versedb.Server):
    """Implementation of the Versedb Cap'n Proto interface."""

    def __init__(self):
        self.store: Dict[bytes, bytes] = {}

    async def add(self, key, value, _context, **kwargs):
        """Add a key-value pair to the store."""
        print(f"➕ Adding key-value pair")
        self.store[key] = value

    async def select(self, key, _context, **kwargs):
        """Select a value by key."""
        print(f"🔍 Selecting value for key")
        if key in self.store:
            _context.results.value = self.store[key]
        else:
            _context.results.value = b""

    async def remove(self, key, _context, **kwargs):
        """Remove a key-value pair from the store."""
        print(f"➖ Removing key-value pair")
        if key in self.store:
            del self.store[key]

    async def selectRange(self, range, _context, **kwargs):
        """Select key-value pairs within a range."""
        print(f"🔍 Selecting range")
        start = range.start
        end = range.end

        # Create a list of matching pairs
        matching_pairs = [
            (key, value) for key, value in self.store.items() if start <= key <= end
        ]

        # Initialize the list with the correct size
        pairs = _context.results.init("pairs", len(matching_pairs))

        # Fill in the pairs
        for i, (key, value) in enumerate(matching_pairs):
            pairs[i].key = key
            pairs[i].value = value

    async def removeRange(self, range, _context, **kwargs):
        """Remove key-value pairs within a range."""
        print(f"➖ Removing range")
        start = range.start
        end = range.end

        # Create a list of matching pairs before removal
        matching_pairs = [
            (key, value) for key, value in self.store.items() if start <= key <= end
        ]

        # Remove the matching pairs
        for key, _ in matching_pairs:
            del self.store[key]

        # Initialize the list with the removed pairs
        pairs = _context.results.init("pairs", len(matching_pairs))

        # Fill in the pairs that were removed
        for i, (key, value) in enumerate(matching_pairs):
            pairs[i].key = key
            pairs[i].value = value

    async def helloworld(self, input, _context, **kwargs):
        """Simple hello world method."""
        print(f"👋 Hello world request with input: {input}")
        _context.results.output = f"Hello, {input}!"


async def new_connection(stream):
    print("🔌 New client connected!")
    server = capnp.TwoPartyServer(stream, bootstrap=VersedbImpl())
    await server.on_disconnect()
    print("🔌 Client disconnected")


def parse_args():
    parser = argparse.ArgumentParser(
        usage="Runs the Versedb server at the given address/port"
    )
    parser.add_argument(
        "--host", default="localhost", help="Host address (default: localhost)"
    )
    parser.add_argument(
        "--port", type=int, default=8000, help="Port number (default: 8000)"
    )
    return parser.parse_args()


async def main():
    args = parse_args()
    print(f"🚀 Server running at {args.host}:{args.port}")
    print("⭐ Ready to accept connections...")
    server = await capnp.AsyncIoStream.create_server(
        new_connection, args.host, args.port
    )
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(capnp.run(main()))
