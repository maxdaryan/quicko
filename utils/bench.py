"""Quicko2 Benchmarking Utility.

Measures latency and throughput for the messaging core.
"""

import time
import statistics


def bench_encrypt_decrypt():
    """Benchmark encryption/decryption throughput."""
    try:
        import quicko2_core
    except ImportError:
        print("quicko2_core not available. Build with: cd core-ffi && maturin develop")
        return

    client = quicko2_core.QuickoClient("ws://localhost:9900")
    session = client.create_session()
    print(f"Session: {session.display_name} ({session.session_id[:8]}...)")

    # TODO: Add encryption benchmarks once the bridge exposes crypto functions
    print("Benchmark placeholder — crypto functions not yet exposed to Python")


def main():
    print("=== Quicko2 Benchmarks ===\n")
    bench_encrypt_decrypt()


if __name__ == "__main__":
    main()
