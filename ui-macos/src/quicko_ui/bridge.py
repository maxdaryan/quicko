"""Bridge between Python UI and Rust core via PyO3."""

from PyQt6.QtCore import QObject, pyqtSignal


class RustBridge(QObject):
    """Thread-safe bridge between Rust core and Qt UI.
    
    In production, this imports the `quicko2_core` PyO3 module.
    Falls back to a mock implementation for UI development.
    """

    message_received = pyqtSignal(str, str, str, float)  # id, sender, text, timestamp
    connection_changed = pyqtSignal(bool)
    peer_joined = pyqtSignal(str, str)  # session_id, display_name
    peer_left = pyqtSignal(str)  # session_id

    def __init__(self, server_url: str = "ws://127.0.0.1:9900"):
        super().__init__()
        self._server_url = server_url
        self._client = None
        self._connected = False

        try:
            import quicko2_core
            self._client = quicko2_core.QuickoClient(server_url)
        except ImportError:
            print("[bridge] quicko2_core not available, using mock mode")

    def create_session(self):
        """Create a new ephemeral session. Returns (session_id, display_name, invite_code)."""
        if self._client:
            info = self._client.create_session()
            return info.session_id, info.display_name, info.invite_code

        # Mock fallback
        import uuid
        return uuid.uuid4().hex[:32], "Swift Falcon #A1B2", "KBQW-E3TU"

    def send_message(self, recipient_id: str, text: str) -> str:
        """Send an encrypted message."""
        if self._client:
            return self._client.send_message(recipient_id, text)
        return "mock-msg-id"

    def destroy_session(self):
        """Destroy session and zeroize all data."""
        if self._client:
            self._client.destroy_session()
        self._connected = False

    @property
    def is_connected(self) -> bool:
        if self._client:
            return self._client.is_connected()
        return self._connected
