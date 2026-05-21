"""Chat input widget with send button."""

from PyQt6.QtWidgets import (
    QWidget, QHBoxLayout, QTextEdit, QPushButton,
    QGraphicsDropShadowEffect, QFrame,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QColor

from ..theme import theme_manager


class ChatInput(QWidget):
    """Chat input area with multi-line text input and send button."""

    message_submitted = pyqtSignal(str)

    def __init__(self):
        super().__init__()
        self.setFixedHeight(84)
        self._setup_ui()

    def _setup_ui(self):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(20, 12, 20, 20)
        layout.setSpacing(12)

        # Container for the input to give it a border-less look within a surface
        self.input_container = QFrame()
        self.input_container.setObjectName("InputContainer")
        self.input_container.setStyleSheet(f"""
            #InputContainer {{
                background-color: {theme_manager.color('bg_input')};
                border: 1px solid {theme_manager.color('border')};
                border-radius: 20px;
            }}
        """)
        
        container_layout = QHBoxLayout(self.input_container)
        container_layout.setContentsMargins(4, 4, 4, 4)
        container_layout.setSpacing(8)

        # Attachment button
        self.attach_btn = QPushButton("+")
        self.attach_btn.setFixedSize(32, 32)
        self.attach_btn.setProperty("secondary", "true")
        self.attach_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.attach_btn.setStyleSheet("border-radius: 16px; font-size: 18px; font-weight: 400;")
        container_layout.addWidget(self.attach_btn)

        # Text input
        self.text_input = QTextEdit()
        self.text_input.setPlaceholderText("Message")
        self.text_input.setFrameShape(QFrame.Shape.NoFrame)
        self.text_input.setStyleSheet("background: transparent; border: none; padding: 4px; font-size: 14px;")
        self.text_input.installEventFilter(self)
        container_layout.addWidget(self.text_input)

        # Send button
        self.send_btn = QPushButton("↑") # Mac-style arrow
        self.send_btn.setFixedSize(32, 32)
        self.send_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.send_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {theme_manager.color('accent')};
                color: white;
                border-radius: 16px;
                font-size: 18px;
                font-weight: 800;
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('accent_hover')};
            }}
        """)
        self.send_btn.clicked.connect(self._on_send)
        container_layout.addWidget(self.send_btn, alignment=Qt.AlignmentFlag.AlignBottom)

        layout.addWidget(self.input_container)

    def _on_send(self):
        text = self.text_input.toPlainText().strip()
        if text:
            self.message_submitted.emit(text)
            self.text_input.clear()
            self.text_input.setFocus()

    def eventFilter(self, source, event):
        from PyQt6.QtCore import QEvent
        if source is self.text_input and event.type() == QEvent.Type.KeyPress:
            if event.key() == Qt.Key.Key_Return and not (event.modifiers() & Qt.KeyboardModifier.ShiftModifier):
                self._on_send()
                return True
        return super().eventFilter(source, event)
