"""Connection status bar widget."""

from PyQt6.QtWidgets import QStatusBar, QLabel, QHBoxLayout, QWidget
from PyQt6.QtCore import Qt
from ..theme import theme_manager

class ConnectionStatusBar(QStatusBar):
    """Status bar showing connection state with colored indicator dot."""

    def __init__(self):
        super().__init__()
        self.setFixedHeight(28)
        
        self.current_state = "disconnected"
        self.current_message = "No active session"
        
        # Left side: Connection status
        self.status_container = QWidget()
        status_layout = QHBoxLayout(self.status_container)
        status_layout.setContentsMargins(12, 0, 8, 0)
        status_layout.setSpacing(6)
        
        self.dot = QLabel("[ ]")
        status_layout.addWidget(self.dot)

        self.status_label = QLabel(self.current_message)
        status_layout.addWidget(self.status_label)
        
        self.addWidget(self.status_container)

        # Right side: Security indicator
        self.security_label = QLabel("E2E Encrypted")
        self.addPermanentWidget(self.security_label)

        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.setStyleSheet(f"""
            QStatusBar {{
                background-color: {theme_manager.color('bg_secondary')};
                border-top: 1px solid {theme_manager.color('border')};
            }}
        """)
        self.status_container.setStyleSheet("background: transparent;")
        self.security_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 10px; "
            f"background: transparent; padding-right: 12px; "
            f"letter-spacing: 0.5px;"
        )
        self.set_status(self.current_state, self.current_message)

    def set_status(self, state: str, message: str = ""):
        self.current_state = state
        self.current_message = message
        
        status_colors = {
            "connected": theme_manager.color('success'),
            "connecting": theme_manager.color('warning'),
            "disconnected": theme_manager.color('text_muted'),
            "error": theme_manager.color('error'),
        }
        
        status_dots = {
            "connected": "[+]",
            "connecting": "[~]",
            "disconnected": "[ ]",
            "error": "[!]",
        }
        
        color = status_colors.get(state, theme_manager.color('text_muted'))
        dot_char = status_dots.get(state, "[ ]")
        
        self.dot.setText(dot_char)
        self.dot.setStyleSheet(
            f"font-size: 10px; color: {color}; background: transparent;"
        )
        self.status_label.setStyleSheet(
            f"color: {color}; font-size: 11px; background: transparent;"
        )
        self.status_label.setText(message)
