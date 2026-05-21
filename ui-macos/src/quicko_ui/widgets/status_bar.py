"""Connection status bar widget."""

from PyQt6.QtWidgets import QStatusBar, QLabel, QHBoxLayout, QWidget, QFrame
from PyQt6.QtCore import Qt
from ..theme import theme_manager

class ConnectionStatusBar(QStatusBar):
    """Status bar showing connection state with colored indicator dot."""

    def __init__(self):
        super().__init__()
        self.setFixedHeight(32)
        self.setSizeGripEnabled(False)
        
        self.current_state = "disconnected"
        self.current_message = "Ready"
        
        container = QWidget()
        layout = QHBoxLayout(container)
        layout.setContentsMargins(16, 0, 16, 0)
        layout.setSpacing(8)
        
        # Indicator Dot
        self.dot = QFrame()
        self.dot.setFixedSize(8, 8)
        self.dot.setObjectName("StatusDot")
        layout.addWidget(self.dot)

        self.status_label = QLabel(self.current_message)
        self.status_label.setStyleSheet("font-size: 11px; font-weight: 500;")
        layout.addWidget(self.status_label)
        
        self.addWidget(container)

        # Right side: Security indicator
        self.security_label = QLabel("🔒 E2E Encrypted")
        self.security_label.setStyleSheet(f"color: {theme_manager.color('text_muted')}; font-size: 10px; margin-right: 12px;")
        self.addPermanentWidget(self.security_label)

        self.set_status(self.current_state, self.current_message)
        theme_manager.theme_changed.connect(self._on_theme_changed)

    def _on_theme_changed(self):
        self.set_status(self.current_state, self.current_message)

    def set_status(self, state: str, message: str = ""):
        self.current_state = state
        self.current_message = message or "Ready"
        
        colors = {
            "connected": theme_manager.color('success'),
            "connecting": theme_manager.color('warning'),
            "disconnected": theme_manager.color('text_muted'),
            "error": theme_manager.color('error'),
        }
        
        color = colors.get(state, theme_manager.color('text_muted'))
        self.dot.setStyleSheet(f"background-color: {color}; border-radius: 4px;")
        self.status_label.setStyleSheet(f"color: {color}; font-size: 11px; font-weight: 500;")
        self.status_label.setText(self.current_message)
        
        self.setStyleSheet(f"""
            QStatusBar {{
                background-color: {theme_manager.color('bg_primary')};
                border-top: 1px solid {theme_manager.color('border')};
            }}
        """)
