"""Session panel — create or join a session."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit, QFrame, QGraphicsDropShadowEffect,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QColor

from ..theme import theme_manager, MONO_FONT_STACK


class SessionPanel(QWidget):
    """Panel for creating or joining an ephemeral session."""

    session_created = pyqtSignal(str, str, str)  # session_id, display_name, invite_code
    session_joined = pyqtSignal(str)  # invite_code

    def __init__(self):
        super().__init__()
        self._setup_ui()
        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.title_label.setStyleSheet(
            f"color: {theme_manager.color('accent')}; "
            f"font-size: 40px; font-weight: 800; "
            f"letter-spacing: -1px; background: transparent;"
        )
        self.glow.setColor(QColor(theme_manager.color('accent')))
        
        self.subtitle.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 13px; "
            f"letter-spacing: 2px; background: transparent;"
        )
        
        self.create_btn.setStyleSheet(f"""
            QPushButton {{
                font-size: 15px;
                font-weight: 600;
                padding: 0 24px;
                border-radius: 12px;
                background-color: {theme_manager.color('accent')};
                color: #FFFFFF;
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('accent_hover')};
            }}
            QPushButton:pressed {{
                background-color: {theme_manager.color('accent_pressed')};
            }}
        """)
        self.btn_glow.setColor(QColor(theme_manager.color('accent')))
        
        self.line_left.setStyleSheet(f"color: {theme_manager.color('border')};")
        self.or_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 12px; "
            f"padding: 0 16px; background: transparent;"
        )
        self.line_right.setStyleSheet(f"color: {theme_manager.color('border')};")
        
        self.invite_input.setStyleSheet(f"""
            QLineEdit {{
                font-size: 16px;
                font-family: {MONO_FONT_STACK};
                letter-spacing: 3px;
                background-color: {theme_manager.color('bg_input')};
                border: 1px solid {theme_manager.color('border')};
                border-radius: 10px;
                padding: 0 16px;
                color: {theme_manager.color('text_primary')};
            }}
            QLineEdit:focus {{
                border-color: {theme_manager.color('accent')};
                background-color: {theme_manager.color('bg_surface')};
            }}
        """)
        
        self.join_btn.setStyleSheet(f"""
            QPushButton {{
                font-size: 15px;
                font-weight: 600;
                padding: 0 24px;
                border-radius: 12px;
                background-color: {theme_manager.color('bg_tertiary')};
                color: {theme_manager.color('text_primary')};
                border: 1px solid {theme_manager.color('border')};
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('bg_hover')};
                border-color: {theme_manager.color('accent')};
            }}
        """)
        
        self.footer.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 11px; "
            f"background: transparent; line-height: 1.5;"
        )

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(20)
        layout.setContentsMargins(48, 48, 48, 48)

        layout.addStretch(2)

        # Logo / Title with glow
        self.title_label = QLabel("Quicko")
        self.title_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        
        self.glow = QGraphicsDropShadowEffect()
        self.glow.setBlurRadius(30)
        self.glow.setOffset(0, 0)
        self.title_label.setGraphicsEffect(self.glow)
        layout.addWidget(self.title_label)

        self.subtitle = QLabel("Ephemeral  ·  Encrypted  ·  Fast")
        self.subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.subtitle)

        layout.addSpacing(40)

        # Create session button — prominent
        self.create_btn = QPushButton("Create New Session")
        self.create_btn.setFixedHeight(52)
        self.create_btn.setFixedWidth(320)
        self.create_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        
        # Button glow
        self.btn_glow = QGraphicsDropShadowEffect()
        self.btn_glow.setBlurRadius(24)
        self.btn_glow.setOffset(0, 4)
        self.create_btn.setGraphicsEffect(self.btn_glow)
        self.create_btn.clicked.connect(self._on_create)
        layout.addWidget(self.create_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        # Divider
        divider_layout = QHBoxLayout()
        divider_layout.setContentsMargins(60, 8, 60, 8)
        
        self.line_left = QFrame()
        self.line_left.setFrameShape(QFrame.Shape.HLine)
        divider_layout.addWidget(self.line_left)

        self.or_label = QLabel("or")
        divider_layout.addWidget(self.or_label)

        self.line_right = QFrame()
        self.line_right.setFrameShape(QFrame.Shape.HLine)
        divider_layout.addWidget(self.line_right)
        layout.addLayout(divider_layout)

        # Join session input
        self.invite_input = QLineEdit()
        self.invite_input.setPlaceholderText("Enter invite code...")
        self.invite_input.setFixedHeight(48)
        self.invite_input.setFixedWidth(320)
        self.invite_input.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.invite_input.returnPressed.connect(self._on_join)
        layout.addWidget(self.invite_input, alignment=Qt.AlignmentFlag.AlignCenter)

        self.join_btn = QPushButton("Join Session")
        self.join_btn.setFixedHeight(52)
        self.join_btn.setFixedWidth(320)
        self.join_btn.setProperty("cssClass", "secondary")
        self.join_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.join_btn.clicked.connect(self._on_join)
        layout.addWidget(self.join_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addStretch(3)

        # Footer
        self.footer = QLabel("All messages are end-to-end encrypted and ephemeral.\nNothing is stored. Nothing is logged.")
        self.footer.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.footer)

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
