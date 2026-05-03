"""Chat input widget with send button."""

from PyQt6.QtWidgets import (
    QWidget, QHBoxLayout, QTextEdit, QPushButton,
)
from PyQt6.QtCore import Qt, pyqtSignal

from ..theme import COLORS


class ChatInput(QWidget):
    """Chat input area with multi-line text input and send button."""

    message_submitted = pyqtSignal(str)

    def __init__(self):
        super().__init__()
        self.setFixedHeight(72)
        self.setStyleSheet(f"""
            QWidget {{
                background-color: {COLORS["bg_secondary"]};
                border-top: 1px solid {COLORS["border"]};
            }}
        """)
        self._setup_ui()

    def _setup_ui(self):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(8)

        # Text input
        self.text_input = QTextEdit()
        self.text_input.setPlaceholderText("Type a message...")
        self.text_input.setFixedHeight(44)
        self.text_input.setStyleSheet(f"""
            QTextEdit {{
                background-color: {COLORS["bg_input"]};
                border: 1px solid {COLORS["border"]};
                border-radius: 22px;
                padding: 8px 16px;
                font-size: 14px;
            }}
            QTextEdit:focus {{
                border-color: {COLORS["border_focus"]};
            }}
        """)
        self.text_input.setVerticalScrollBarPolicy(
            Qt.ScrollBarPolicy.ScrollBarAlwaysOff
        )
        layout.addWidget(self.text_input)

        # Send button
        send_btn = QPushButton("↑")
        send_btn.setFixedSize(44, 44)
        send_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {COLORS["accent"]};
                color: white;
                border-radius: 22px;
                font-size: 20px;
                font-weight: bold;
            }}
            QPushButton:hover {{
                background-color: {COLORS["accent_hover"]};
            }}
        """)
        send_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        send_btn.clicked.connect(self._on_send)
        layout.addWidget(send_btn)

    def _on_send(self):
        text = self.text_input.toPlainText().strip()
        if text:
            self.message_submitted.emit(text)
            self.text_input.clear()
            self.text_input.setFocus()

    def keyPressEvent(self, event):
        """Handle Enter to send, Shift+Enter for newline."""
        if event.key() == Qt.Key.Key_Return and not event.modifiers() & Qt.KeyboardModifier.ShiftModifier:
            self._on_send()
        else:
            super().keyPressEvent(event)
