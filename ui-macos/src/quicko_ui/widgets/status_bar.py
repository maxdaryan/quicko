"""Connection status bar widget."""

from PyQt6.QtWidgets import QStatusBar, QLabel
from PyQt6.QtCore import Qt
from ..theme import COLORS

STATUS_COLORS = {
    "connected": COLORS["success"],
    "connecting": COLORS["warning"],
    "disconnected": COLORS["text_muted"],
    "error": COLORS["error"],
}


class ConnectionStatusBar(QStatusBar):
    """Status bar showing connection state."""

    def __init__(self):
        super().__init__()
        self.dot = QLabel("●")
        self.dot.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 10px; background: transparent;")
        self.addWidget(self.dot)

        self.status_label = QLabel("No active session")
        self.status_label.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 12px; background: transparent;")
        self.addWidget(self.status_label)

        self.set_status("disconnected", "No active session")

    def set_status(self, state: str, message: str = ""):
        color = STATUS_COLORS.get(state, COLORS["text_muted"])
        self.dot.setStyleSheet(f"color: {color}; font-size: 10px; background: transparent;")
        self.status_label.setText(message)
