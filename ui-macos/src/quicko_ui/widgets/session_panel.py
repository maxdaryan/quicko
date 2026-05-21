"""Session panel — create or join a session."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit, QFrame,
)
from PyQt6.QtCore import Qt, pyqtSignal
from ..theme import theme_manager, MONO_FONT_STACK


class SessionPanel(QWidget):
    """Panel for creating or joining an ephemeral session."""

    session_created = pyqtSignal(str, str, str)
    session_joined = pyqtSignal(str)

    def __init__(self):
        super().__init__()
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(16)
        layout.setContentsMargins(60, 60, 60, 60)

        layout.addStretch(1)

        # Title
        title = QLabel("Quicko")
        title.setObjectName("WelcomeLogo")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(title)

        tagline = QLabel("Ephemeral · Encrypted · Fast")
        tagline.setObjectName("WelcomeTagline")
        tagline.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(tagline)

        layout.addSpacing(40)

        # Create
        self.create_btn = QPushButton("Create New Session")
        self.create_btn.setFixedSize(300, 52)
        self.create_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.create_btn.clicked.connect(self._on_create)
        layout.addWidget(self.create_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        # Divider
        divider = QLabel("— or —")
        divider.setStyleSheet(f"color: {theme_manager.color('text_muted')}; font-size: 13px; font-weight: 600;")
        layout.addWidget(divider, alignment=Qt.AlignmentFlag.AlignCenter)

        # Join
        self.invite_input = QLineEdit()
        self.invite_input.setPlaceholderText("Enter invite code")
        self.invite_input.setFixedSize(300, 48)
        self.invite_input.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.invite_input.setStyleSheet(f"font-family: {MONO_FONT_STACK}; letter-spacing: 2px;")
        layout.addWidget(self.invite_input, alignment=Qt.AlignmentFlag.AlignCenter)

        self.join_btn = QPushButton("Join Session")
        self.join_btn.setProperty("secondary", "true")
        self.join_btn.setFixedSize(300, 52)
        self.join_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.join_btn.clicked.connect(self._on_join)
        layout.addWidget(self.join_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addStretch(1)

        # Footer
        footer = QLabel("End-to-end encrypted. No logs. No trace.")
        footer.setStyleSheet(f"color: {theme_manager.color('text_muted')}; font-size: 11px;")
        footer.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(footer)

    def _on_create(self):
        """Create a new ephemeral session."""
        import uuid
        session_id = uuid.uuid4().hex[:32]
        # In a real app, these would come from the core
        display_name = "Swift Falcon #A1B2"
        invite_code = "KBQW-E3TU"
        self.session_created.emit(session_id, display_name, invite_code)

    def _on_join(self):
        """Join an existing session by invite code."""
        code = self.invite_input.text().strip().upper()
        if len(code) >= 4:
            self.session_joined.emit(code)
