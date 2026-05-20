"""Chat input widget with send button."""

from PyQt6.QtWidgets import (
    QWidget, QHBoxLayout, QTextEdit, QPushButton,
    QGraphicsDropShadowEffect,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QColor

from ..theme import theme_manager


class ChatInput(QWidget):
    """Chat input area with multi-line text input and send button."""

    message_submitted = pyqtSignal(str)

    def __init__(self):
        super().__init__()
        self.setFixedHeight(76)
        self._setup_ui()
        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.setStyleSheet(f"""
            QWidget {{
                background-color: {theme_manager.color('bg_secondary')};
                border-top: 1px solid {theme_manager.color('border')};
            }}
        """)
        self.attach_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {theme_manager.color('bg_tertiary')};
                color: {theme_manager.color('text_secondary')};
                border-radius: 19px;
                font-size: 20px;
                font-weight: 500;
                border: 1px solid {theme_manager.color('border')};
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('bg_hover')};
                color: {theme_manager.color('text_primary')};
                border-color: {theme_manager.color('accent')};
            }}
        """)
        self.text_input.setStyleSheet(f"""
            QTextEdit {{
                background-color: {theme_manager.color('bg_input')};
                border: 1px solid {theme_manager.color('border')};
                border-radius: 22px;
                padding: 8px 18px;
                font-size: 14px;
                color: {theme_manager.color('text_primary')};
            }}
            QTextEdit:focus {{
                border-color: {theme_manager.color('border_focus')};
                background-color: {theme_manager.color('bg_surface')};
            }}
        """)
        self.send_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {theme_manager.color('accent')};
                color: white;
                border-radius: 22px;
                font-size: 20px;
                font-weight: bold;
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('accent_hover')};
            }}
            QPushButton:pressed {{
                background-color: {theme_manager.color('accent_pressed')};
            }}
        """)
        # Update glow effect color
        self.glow.setColor(QColor(theme_manager.color('accent')))

    def _setup_ui(self):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(16, 14, 16, 14)
        layout.setSpacing(10)

        # Attachment button
        self.attach_btn = QPushButton("+")
        self.attach_btn.setFixedSize(38, 38)
        self.attach_btn.setToolTip("Attach file")
        self.attach_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        layout.addWidget(self.attach_btn)

        # Text input
        self.text_input = QTextEdit()
        self.text_input.setPlaceholderText("Type a message...")
        self.text_input.setFixedHeight(44)
        self.text_input.setVerticalScrollBarPolicy(
            Qt.ScrollBarPolicy.ScrollBarAlwaysOff
        )
        # Install event filter so we capture key presses inside the QTextEdit.
        # Overriding keyPressEvent on the container QWidget doesn't work because
        # QTextEdit consumes its own key events before they bubble up.
        self.text_input.installEventFilter(self)
        layout.addWidget(self.text_input)

        # Send button with glow
        self.send_btn = QPushButton(">")
        self.send_btn.setFixedSize(44, 44)
        
        # Glow effect
        self.glow = QGraphicsDropShadowEffect()
        self.glow.setBlurRadius(20)
        self.glow.setOffset(0, 0)
        self.send_btn.setGraphicsEffect(self.glow)
        self.send_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.send_btn.clicked.connect(self._on_send)
        layout.addWidget(self.send_btn)

    def _on_send(self):
        text = self.text_input.toPlainText().strip()
        if text:
            self.message_submitted.emit(text)
            self.text_input.clear()
            self.text_input.setFocus()

    def eventFilter(self, source, event):
        """Intercept key events on the text input to implement Enter-to-send."""
        from PyQt6.QtCore import QEvent
        if source is self.text_input and event.type() == QEvent.Type.KeyPress:
            modifiers = event.modifiers()
            if (
                event.key() == Qt.Key.Key_Return
                and not (modifiers & Qt.KeyboardModifier.ShiftModifier)
            ):
                self._on_send()
                return True  # Consume event — don't insert newline
        return super().eventFilter(source, event)
