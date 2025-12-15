#!/usr/bin/env python3
"""
Test script to verify HMAC signature computation against Polymarket's Python client.
Uses the exact same inputs from our Rust logs to compare outputs.
"""

import base64
import hmac
import hashlib

# From the Rust logs:
SECRET_BASE64 = "L95OWCgbprfwozNeNUYnvU7A91iM-EH3uDZ6RTZYa-w="
TIMESTAMP = "1765824355"
METHOD = "POST"
PATH = "/order"
BODY = '{"order":{"salt":327896216,"maker":"0xefa7cd2e9bfa38f04af95df90da90b194e4ed191","signer":"0xefa7cd2e9bfa38f04af95df90da90b194e4ed191","taker":"0x0000000000000000000000000000000000000000","tokenId":"59123046651639406043770531564026866824584320057748742767920960374229735119462","makerAmount":"85000","takerAmount":"100000","expiration":"0","nonce":"0","feeRateBps":"0","side":"BUY","signatureType":0,"signature":"0x6c96640200869044e2d77ffb01e69209a415d4818a122f297362248ca69246a91c83a52829dd5946e8d0ba36c8afef55592d43b0a8b19327e486846b6f5ec0481c"},"owner":"1a846e10-1906-c373-b2b9-c853e39a334a","orderType":"GTC"}'

def build_hmac_signature_python_client(secret: str, timestamp: int, method: str, request_path: str, body: str = "") -> str:
    """
    This is the EXACT implementation from Polymarket's py-clob-client
    See: https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/signing/hmac.py
    """
    message = str(timestamp) + str(method) + str(request_path)
    if body:
        # Python client does: str(body).replace("'", '"')
        # But since we're passing a JSON string already, this shouldn't matter
        message += str(body).replace("'", '"')

    print(f"=== Python Client HMAC ===")
    print(f"Secret (base64): {secret}")

    # Decode secret
    decoded_secret = base64.urlsafe_b64decode(secret)
    print(f"Secret decoded: {len(decoded_secret)} bytes")
    print(f"Secret bytes (hex): {decoded_secret.hex()}")

    print(f"Message: {message[:100]}...")
    print(f"Message bytes (hex): {message.encode().hex()}")

    # Compute HMAC
    hmac_obj = hmac.new(decoded_secret, message.encode(), hashlib.sha256)
    result_bytes = hmac_obj.digest()
    print(f"HMAC result bytes (hex): {result_bytes.hex()}")

    # Encode result
    signature = base64.urlsafe_b64encode(result_bytes).decode("utf-8")
    print(f"Final signature (base64): {signature}")
    print(f"===========================")

    return signature

def main():
    print("Testing HMAC computation with Polymarket Python client logic\n")

    # Compute using Python client logic
    sig = build_hmac_signature_python_client(
        SECRET_BASE64,
        TIMESTAMP,
        METHOD,
        PATH,
        BODY
    )

    print(f"\n\nRust signature:   fPOEaxvya1QbVkoRPNrv-kxPjv16Ni_mxrqMgCzm-hc")
    print(f"Python signature: {sig}")
    print(f"Match: {sig.rstrip('=') == 'fPOEaxvya1QbVkoRPNrv-kxPjv16Ni_mxrqMgCzm-hc'}")

if __name__ == "__main__":
    main()
