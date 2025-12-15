#!/usr/bin/env python3
"""
Test order submission using the official Polymarket Python client.
This will help verify if our Rust implementation is correct by comparing against
a known working implementation.

Install: pip install py-clob-client
"""

import os
import json
from dotenv import load_dotenv

# Load environment variables
load_dotenv('.env.local')

PRIVATE_KEY = os.getenv('TRADING_PRIVATE_KEY')

if not PRIVATE_KEY:
    print("ERROR: TRADING_PRIVATE_KEY not found in .env.local")
    exit(1)

print(f"Private key loaded (length: {len(PRIVATE_KEY)})")

try:
    from py_clob_client.client import ClobClient
    from py_clob_client.clob_types import OrderArgs, OrderType
    from py_clob_client.order_builder.constants import BUY
except ImportError:
    print("ERROR: py-clob-client not installed. Run: pip install py-clob-client")
    exit(1)

# Polymarket mainnet
HOST = "https://clob.polymarket.com"
CHAIN_ID = 137  # Polygon mainnet

def main():
    print("\n=== Initializing Polymarket Client ===")

    # Create client with private key
    client = ClobClient(
        HOST,
        key=PRIVATE_KEY,
        chain_id=CHAIN_ID,
    )

    print(f"Client created")
    print(f"Address: {client.get_address()}")

    # Get or create API credentials
    print("\n=== Getting API Credentials ===")
    try:
        # Try to derive existing API key
        client.set_api_creds(client.derive_api_key())
        print("Derived existing API key")
    except Exception as e:
        print(f"Could not derive API key: {e}")
        print("Creating new API key...")
        client.set_api_creds(client.create_api_key())
        print("Created new API key")

    creds = client.creds
    print(f"API Key: {creds.api_key}")
    print(f"Secret: {creds.api_secret}")
    print(f"Passphrase: {creds.api_passphrase}")

    # Test a simple authenticated endpoint first
    print("\n=== Testing Authenticated Endpoint ===")
    try:
        # Get open orders (should be empty if no orders placed)
        orders = client.get_orders()
        print(f"Open orders: {len(orders) if orders else 0}")
    except Exception as e:
        print(f"ERROR getting orders: {e}")
        print("This suggests L2 auth is failing!")
        return

    # Now try to create an order (but don't submit yet - just build it)
    print("\n=== Building Test Order ===")

    # Use a real token ID from your market
    TOKEN_ID = "59123046651639406043770531564026866824584320057748742767920960374229735119462"

    try:
        order_args = OrderArgs(
            price=0.85,
            size=0.1,
            side=BUY,
            token_id=TOKEN_ID,
        )

        print(f"Order args: price=0.85, size=0.1, side=BUY")

        # Build the signed order
        signed_order = client.create_order(order_args)

        print(f"\nSigned order built successfully!")
        print(f"Order dict: {json.dumps(signed_order.dict(), indent=2)}")

        # Now try to actually submit it
        print("\n=== Submitting Order ===")
        result = client.post_order(signed_order, OrderType.GTC)
        print(f"Result: {result}")

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()
