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
        self.is_sent = is_sent
        self._setup(text, is_sent, timestamp)
        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        bg = theme_manager.color('message_sent') if self.is_sent else theme_manager.color('message_received')
        
        # Asymmetric border radius for a modern chat bubble look
        if self.is_sent:
            radius = "16px 16px 4px 16px"
        else:
            radius = "16px 16px 16px 4px"

        self.setStyleSheet(f"""
            QFrame {{
                background-color: {bg};
                border-radius: {radius};
                padding: 10px 16px;
                margin: 2px {"48px 2px 12px" if self.is_sent else "12px 2px 48px"};
                border: 1px solid {theme_manager.color('border_subtle')};
            }}
        """)
        
        self.msg_label.setStyleSheet(
            f"color: {theme_manager.color('text_primary')}; font-size: 14px; "
            f"background: transparent; line-height: 1.4;"
        )
        self.time_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 10px; "
            f"background: transparent;"
        )

    def _setup(self, text: str, is_sent: bool, timestamp: str):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        self.msg_label = QLabel(text)
        self.msg_label.setWordWrap(True)
        layout.addWidget(self.msg_label)

        if not timestamp:
            timestamp = QDateTime.currentDateTime().toString("hh:mm")

        self.time_label = QLabel(timestamp)
        self.time_label.setAlignment(
            Qt.AlignmentFlag.AlignRight if is_sent else Qt.AlignmentFlag.AlignLeft
        )
        layout.addWidget(self.time_label)


class MessageList(QScrollArea):
    """Scrollable message list with smooth auto-scroll."""

    def __init__(self):
        super().__init__()
        self.setWidgetResizable(True)
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        
        self.container = QWidget()
        self.layout = QVBoxLayout(self.container)
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.layout.setSpacing(6)
        self.layout.setContentsMargins(12, 12, 12, 12)
        self.layout.addStretch()
        self.setWidget(self.container)

        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.setStyleSheet(
            f"background-color: {theme_manager.color('bg_primary')}; border: none;"
        )
        self.container.setStyleSheet("background: transparent;")

    def add_sent_message(self, text: str):
        bubble = MessageBubble(text, is_sent=True)
        self.layout.insertWidget(self.layout.count() - 1, bubble)
        # Use a timer to scroll after the layout updates
        QTimer.singleShot(50, self._scroll_to_bottom)

    def add_received_message(self, text: str, sender: str = ""):
        bubble = MessageBubble(text, is_sent=False)
        self.layout.insertWidget(self.layout.count() - 1, bubble)
        QTimer.singleShot(50, self._scroll_to_bottom)

    def _scroll_to_bottom(self):
        sb = self.verticalScrollBar()
        # Ensure scrollbar exists and can scroll
        if sb.maximum() > 0:
            self.anim = QPropertyAnimation(sb, b"value")
            self.anim.setDuration(300)
            self.anim.setStartValue(sb.value())
            self.anim.setEndValue(sb.maximum())
            self.anim.start()
