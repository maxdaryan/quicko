"""Quicko2 Key Generation Helper.

Development utility for generating test keypairs and invite codes.
"""

import secrets
import base64


def generate_session_id() -> str:
    """Generate a 128-bit random session ID (hex)."""
    return secrets.token_hex(16)


def generate_invite_code() -> str:
    """Generate an 8-character invite code."""
    raw = secrets.token_bytes(5)
    # Simple base32-like encoding
    chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    code = ""
    val = int.from_bytes(raw, "big")
    for _ in range(8):
        code = chars[val % 32] + code
        val //= 32
    return f"{code[:4]}-{code[4:]}"


def generate_keypair_hex() -> tuple:
    """Generate a random 32-byte key pair (hex encoded)."""
    private = secrets.token_hex(32)
    # In production, public key is derived via X25519
    public = secrets.token_hex(32)  # Placeholder
    return private, public


def main():
    print("=== Quicko2 Key Generator ===\n")

    sid = generate_session_id()
    print(f"Session ID:  {sid}")

    code = generate_invite_code()
    print(f"Invite Code: {code}")

    priv, pub = generate_keypair_hex()
    print(f"Private Key: {priv[:16]}...")
    print(f"Public Key:  {pub[:16]}...")


if __name__ == "__main__":
    main()
