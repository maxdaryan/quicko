"""Message bubble widgets for the chat view."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QLabel, QScrollArea, QFrame,
)
from PyQt6.QtCore import Qt, QDateTime
from ..theme import COLORS


class MessageBubble(QFrame):
    """A single message bubble."""

    def __init__(self, text: str, is_sent: bool, timestamp: str = ""):
        super().__init__()
        self._setup(text, is_sent, timestamp)

    def _setup(self, text: str, is_sent: bool, timestamp: str):
        bg = COLORS["message_sent"] if is_sent else COLORS["message_received"]
        align = "right" if is_sent else "left"
        radius = "18px 18px 4px 18px" if is_sent else "18px 18px 18px 4px"

        self.setStyleSheet(f"""
            QFrame {{
                background-color: {bg};
                border-radius: {radius};
                padding: 10px 14px;
                margin: 2px {"48px 2px 12px" if is_sent else "12px 2px 48px"};
            }}
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        msg_label = QLabel(text)
        msg_label.setWordWrap(True)
        msg_label.setStyleSheet(f"color: {COLORS['text_primary']}; font-size: 14px; background: transparent;")
        layout.addWidget(msg_label)

        if not timestamp:
            timestamp = QDateTime.currentDateTime().toString("hh:mm")

        time_label = QLabel(timestamp)
        time_label.setAlignment(
            Qt.AlignmentFlag.AlignRight if is_sent else Qt.AlignmentFlag.AlignLeft
        )
        time_label.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 10px; background: transparent;")
        layout.addWidget(time_label)


class MessageList(QScrollArea):
    """Scrollable message list."""

    def __init__(self):
        super().__init__()
        self.setWidgetResizable(True)
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.setStyleSheet(f"background-color: {COLORS['bg_primary']}; border: none;")

        self.container = QWidget()
        self.layout = QVBoxLayout(self.container)
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.layout.setSpacing(4)
        self.layout.setContentsMargins(8, 8, 8, 8)
        self.layout.addStretch()
        self.setWidget(self.container)

    def add_sent_message(self, text: str):
        bubble = MessageBubble(text, is_sent=True)
        self.layout.insertWidget(self.layout.count() - 1, bubble)
        self._scroll_to_bottom()

    def add_received_message(self, text: str, sender: str = ""):
        bubble = MessageBubble(text, is_sent=False)
        self.layout.insertWidget(self.layout.count() - 1, bubble)
        self._scroll_to_bottom()

    def _scroll_to_bottom(self):
        sb = self.verticalScrollBar()
        sb.setValue(sb.maximum())
