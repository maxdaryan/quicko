"""Session panel — create or join a session."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit, QFrame,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QFont

from ..theme import COLORS


class SessionPanel(QWidget):
    """Panel for creating or joining an ephemeral session."""

    session_created = pyqtSignal(str, str, str)  # session_id, display_name, invite_code
    session_joined = pyqtSignal(str)  # invite_code

    def __init__(self):
        super().__init__()
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(24)
        layout.setContentsMargins(48, 48, 48, 48)

        # Logo / Title
        title = QLabel("Quicko2")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        title.setFont(QFont("SF Pro Display", 32, QFont.Weight.Bold))
        title.setStyleSheet(f"color: {COLORS['accent']}; background: transparent;")
        layout.addWidget(title)

        subtitle = QLabel("Ephemeral. Encrypted. Fast.")
        subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        subtitle.setStyleSheet(f"color: {COLORS['text_secondary']}; font-size: 14px; background: transparent;")
        layout.addWidget(subtitle)

        layout.addSpacing(32)

        # Create session button
        create_btn = QPushButton("Create New Session")
        create_btn.setFixedHeight(48)
        create_btn.setFixedWidth(280)
        create_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        create_btn.clicked.connect(self._on_create)
        layout.addWidget(create_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        # Divider
        divider_layout = QHBoxLayout()
        line_left = QFrame()
        line_left.setFrameShape(QFrame.Shape.HLine)
        line_left.setStyleSheet(f"color: {COLORS['border']};")
        divider_layout.addWidget(line_left)

        or_label = QLabel("OR")
        or_label.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 12px; padding: 0 12px; background: transparent;")
        divider_layout.addWidget(or_label)

        line_right = QFrame()
        line_right.setFrameShape(QFrame.Shape.HLine)
        line_right.setStyleSheet(f"color: {COLORS['border']};")
        divider_layout.addWidget(line_right)
        layout.addLayout(divider_layout)

        # Join session
        self.invite_input = QLineEdit()
        self.invite_input.setPlaceholderText("Enter invite code (e.g., KBQW-E3TU)")
        self.invite_input.setFixedHeight(44)
        self.invite_input.setFixedWidth(280)
        self.invite_input.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.invite_input.setStyleSheet(f"""
            QLineEdit {{
                font-size: 16px;
                font-family: monospace;
                letter-spacing: 2px;
                text-transform: uppercase;
            }}
        """)
        self.invite_input.returnPressed.connect(self._on_join)
        layout.addWidget(self.invite_input, alignment=Qt.AlignmentFlag.AlignCenter)

        join_btn = QPushButton("Join Session")
        join_btn.setFixedHeight(48)
        join_btn.setFixedWidth(280)
        join_btn.setProperty("cssClass", "secondary")
        join_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        join_btn.clicked.connect(self._on_join)
        layout.addWidget(join_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addStretch()

        # Footer
        footer = QLabel("All messages are end-to-end encrypted and ephemeral.\nNothing is stored. Nothing is logged.")
        footer.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 11px; background: transparent;")
        layout.addWidget(footer)

    def _on_create(self):
        """Create a new ephemeral session."""
        # In production, this calls the Rust core via PyO3
        # For now, emit with placeholder values
        import uuid
        session_id = uuid.uuid4().hex[:32]
        display_name = "Swift Falcon #A1B2"
        invite_code = "KBQW-E3TU"
        self.session_created.emit(session_id, display_name, invite_code)

    def _on_join(self):
        """Join an existing session by invite code."""
        code = self.invite_input.text().strip().upper()
        if len(code) >= 4:
            self.session_joined.emit(code)
