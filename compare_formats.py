#!/usr/bin/env python3
"""Compare our Rust output format against Python client format exactly."""

# Python client output (from successful test):
PYTHON_ORDER = {
  "salt": 1297788380,
  "maker": "0xeFa7Cd2E9BFa38F04Af95df90da90B194e4ed191",
  "signer": "0xeFa7Cd2E9BFa38F04Af95df90da90B194e4ed191",
  "taker": "0x0000000000000000000000000000000000000000",
  "tokenId": "59123046651639406043770531564026866824584320057748742767920960374229735119462",
  "makerAmount": "85000",
  "takerAmount": "100000",
  "expiration": "0",
  "nonce": "0",
  "feeRateBps": "0",
  "side": "BUY",
  "signatureType": 0,
  "signature": "0xf17d2cdb17146baacbf404baf3e4857b651d74c89fec74a79fd987ca59e0442572d684f0b1951de5b2746082b6765ade05a3168606f6d3dc24423d1603c34be01b"
}

# Our Rust output (from logs):
RUST_ORDER = {
  "salt": 425011855,
  "maker": "0xeFa7Cd2E9BFa38F04Af95df90da90B194e4ed191",
  "signer": "0xeFa7Cd2E9BFa38F04Af95df90da90B194e4ed191",
  "taker": "0x0000000000000000000000000000000000000000",
  "tokenId": "59123046651639406043770531564026866824584320057748742767920960374229735119462",
  "makerAmount": "85000",
  "takerAmount": "100000",
  "expiration": "0",
  "nonce": "0",
  "feeRateBps": "0",
  "side": "BUY",
  "signatureType": 0,
  "signature": "0x9456c9d3c3a06b227eb067c210fa1221de0455ace43bcd5b3404e3b24cbb1d624e5bc1775909793ad3d9749b1f1f71eebf959e6e92419fc4eb635afe44ed3b921b"
}

print("=== Field-by-field comparison ===\n")

for key in PYTHON_ORDER:
    py_val = PYTHON_ORDER[key]
    rust_val = RUST_ORDER.get(key)

    py_type = type(py_val).__name__
    rust_type = type(rust_val).__name__ if rust_val is not None else "MISSING"

    match = "✅" if py_val == rust_val or (key in ["salt", "signature"]) else "❌"

    # For salt and signature, we just check types match (values will differ)
    if key in ["salt", "signature"]:
        type_match = "✅" if py_type == rust_type else "❌"
        print(f"{key}:")
        print(f"  Python type: {py_type}, Rust type: {rust_type} {type_match}")
        if key == "salt":
            print(f"  (values differ - expected, salt is random)")
        else:
            print(f"  (values differ - expected, signature depends on salt)")
    else:
        print(f"{key}:")
        print(f"  Python: {py_val} ({py_type})")
        print(f"  Rust:   {rust_val} ({rust_type})")
        print(f"  Match: {match}")
    print()

# Check for extra keys
rust_keys = set(RUST_ORDER.keys())
python_keys = set(PYTHON_ORDER.keys())

if rust_keys - python_keys:
    print(f"❌ Extra keys in Rust: {rust_keys - python_keys}")
if python_keys - rust_keys:
    print(f"❌ Missing keys in Rust: {python_keys - rust_keys}")
if rust_keys == python_keys:
    print("✅ Same keys in both")
