"""Message bubble widgets for the chat view."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QLabel, QScrollArea, QFrame,
    QHBoxLayout, QSizePolicy,
)
from PyQt6.QtCore import Qt, QDateTime, QTimer, QPropertyAnimation
from ..theme import theme_manager


class MessageBubble(QFrame):
    """A single message bubble with modern styling."""

    def __init__(self, text: str, is_sent: bool, timestamp: str = ""):
        super().__init__()
        self.setObjectName("MessageBubble")
        self.setProperty("sent", str(is_sent).lower())
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(4)

        self.msg_label = QLabel(text)
        self.msg_label.setObjectName("MessageText")
        self.msg_label.setWordWrap(True)
        self.msg_label.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
        layout.addWidget(self.msg_label)

        if not timestamp:
            timestamp = QDateTime.currentDateTime().toString("HH:mm")

        self.time_label = QLabel(timestamp)
        self.time_label.setObjectName("MessageTime")
        self.time_label.setProperty("sent", str(is_sent).lower())
        self.time_label.setAlignment(
            Qt.AlignmentFlag.AlignRight if is_sent else Qt.AlignmentFlag.AlignLeft
        )
        layout.addWidget(self.time_label)
        
        # Alignment wrapper
        self.wrapper = QWidget()
        wrapper_layout = QHBoxLayout(self.wrapper)
        wrapper_layout.setContentsMargins(0, 0, 0, 0)
        
        if is_sent:
            wrapper_layout.addStretch()
            wrapper_layout.addWidget(self)
        else:
            wrapper_layout.addWidget(self)
            wrapper_layout.addStretch()


class MessageList(QScrollArea):
    """Scrollable message list with smooth auto-scroll."""

    def __init__(self):
        super().__init__()
        self.setWidgetResizable(True)
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.setFrameShape(QFrame.Shape.NoFrame)
        
        self.container = QWidget()
        self.layout = QVBoxLayout(self.container)
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.layout.setSpacing(2)
        self.layout.setContentsMargins(16, 16, 16, 16)
        self.layout.addStretch()
        self.setWidget(self.container)

    def add_sent_message(self, text: str):
        bubble = MessageBubble(text, is_sent=True)
        self.layout.insertWidget(self.layout.count() - 1, bubble.wrapper)
        QTimer.singleShot(10, self._scroll_to_bottom)

    def add_received_message(self, text: str, sender: str = ""):
        bubble = MessageBubble(text, is_sent=False)
        self.layout.insertWidget(self.layout.count() - 1, bubble.wrapper)
        QTimer.singleShot(10, self._scroll_to_bottom)

    def _scroll_to_bottom(self):
        sb = self.verticalScrollBar()
        sb.setValue(sb.maximum())
