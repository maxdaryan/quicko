"""Sidebar widget for Quicko2 — lists active and previous sessions."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
    QScrollArea, QPushButton, QFrame, QLineEdit,
    QGraphicsDropShadowEffect,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QFont, QColor

from ..theme import theme_manager


class SessionItem(QFrame):
    """A single session entry in the sidebar."""
    
    clicked = pyqtSignal(str)  # session_id

    # Deterministic avatar color palette
    AVATAR_COLORS = [
        "#7C6CF7", "#F76C8C", "#6CF7A0", "#F7C96C",
        "#6CC9F7", "#D76CF7", "#F76C6C", "#6CF7E0",
    ]

    def __init__(self, session_id: str, display_name: str, last_msg: str = "", timestamp: str = "", is_active: bool = False):
        super().__init__()
        self.session_id = session_id
        self.display_name = display_name
        self.is_active = is_active
        self.selected = False
        self._setup_ui(display_name, last_msg, timestamp)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        avatar_color = self._avatar_color()
        self.avatar.setStyleSheet(f"""
            QLabel {{
                background-color: {theme_manager.color('avatar_bg')};
                color: {avatar_color};
                border-radius: 19px;
                font-weight: bold;
                font-size: 15px;
                border: 2px solid {avatar_color};
            }}
        """)
        self.name_label.setStyleSheet(
            f"color: {theme_manager.color('text_primary')}; font-weight: 600; "
            f"font-size: 13px; background: transparent;"
        )
        self.last_msg_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; font-size: 11px; "
            f"background: transparent;"
        )
        if hasattr(self, 'time_label'):
            self.time_label.setStyleSheet(
                f"color: {theme_manager.color('text_muted')}; font-size: 10px; "
                f"background: transparent;"
            )
        self.update_selection(self.selected)

    def _avatar_color(self) -> str:
        """Deterministic color based on session ID."""
        idx = sum(ord(c) for c in self.session_id) % len(self.AVATAR_COLORS)
        return self.AVATAR_COLORS[idx]

    def _setup_ui(self, name, last_msg, time):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(12)

        # Avatar circle with accent color
        self.avatar = QLabel(name[0].upper() if name else "?")
        self.avatar.setFixedSize(38, 38)
        self.avatar.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.avatar)

        # Info
        info_layout = QVBoxLayout()
        info_layout.setSpacing(3)
        
        self.name_label = QLabel(name)
        info_layout.addWidget(self.name_label)

        self.last_msg_label = QLabel(last_msg or "No messages yet")
        self.last_msg_label.setMaximumWidth(140)
        info_layout.addWidget(self.last_msg_label)

        layout.addLayout(info_layout)
        layout.addStretch()

        # Time
        if time:
            self.time_label = QLabel(time)
            layout.addWidget(self.time_label, alignment=Qt.AlignmentFlag.AlignTop)

    def update_selection(self, selected: bool):
        self.selected = selected
        if selected:
            self.setStyleSheet(f"""
                QFrame {{
                    background-color: {theme_manager.color('bg_hover')};
                    border-radius: 10px;
                    border-left: 3px solid {theme_manager.color('accent')};
                }}
            """)
        else:
            self.setStyleSheet(f"""
                QFrame {{
                    background-color: transparent;
                    border-radius: 10px;
                    border-left: 3px solid transparent;
                }}
                QFrame:hover {{
                    background-color: {theme_manager.color('sidebar_hover')};
                }}
            """)

    def mousePressEvent(self, event):
        self.clicked.emit(self.session_id)
        super().mousePressEvent(event)


class Sidebar(QWidget):
    """Sidebar containing the session list and 'New Chat' button."""
    
    session_selected = pyqtSignal(str)
    new_chat_requested = pyqtSignal()

    def __init__(self):
        super().__init__()
        self.setFixedWidth(272)
        self.setStyleSheet(
            f"background-color: {theme_manager.color('sidebar_bg')}; "
            f"border-right: 1px solid {theme_manager.color('border')};"
        )
        
        self.items = {}  # session_id -> SessionItem
        self.selected_id = None
        
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(14, 18, 14, 14)
        layout.setSpacing(12)

        # Header
        header = QHBoxLayout()
        header.setContentsMargins(4, 0, 0, 0)
        
        self.title = QLabel("Chats")
        header.addWidget(self.title)
        header.addStretch()
        
        self.new_btn = QPushButton("+")
        self.new_btn.setFixedSize(30, 30)
        self.new_btn.setToolTip("New Chat")
        self.new_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        # Add glow to new chat button
        self.btn_glow = QGraphicsDropShadowEffect()
        self.btn_glow.setBlurRadius(16)
        self.btn_glow.setOffset(0, 0)
        self.new_btn.setGraphicsEffect(self.btn_glow)
        self.new_btn.clicked.connect(self.new_chat_requested.emit)
        header.addWidget(self.new_btn)
        
        layout.addLayout(header)

        # Search Bar
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search chats...")
        self.search_input.setFixedHeight(34)
        self.search_input.textChanged.connect(self._on_search)
        layout.addWidget(self.search_input)

        # Scroll Area for sessions
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        
        self.container = QWidget()
        self.list_layout = QVBoxLayout(self.container)
        self.list_layout.setContentsMargins(0, 4, 0, 0)
        self.list_layout.setSpacing(4)
        self.list_layout.addStretch()
        
        self.scroll.setWidget(self.container)
        layout.addWidget(self.scroll)

        # Footer / Settings
        footer = QHBoxLayout()
        footer.setContentsMargins(4, 0, 0, 0)
        self.settings_btn = QPushButton("≡  Settings")
        self.settings_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.settings_btn.clicked.connect(theme_manager.toggle_theme)
        footer.addWidget(self.settings_btn)
        layout.addLayout(footer)

        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.setStyleSheet(
            f"background-color: {theme_manager.color('sidebar_bg')}; "
            f"border-right: 1px solid {theme_manager.color('border')};"
        )
        self.title.setStyleSheet(
            f"color: {theme_manager.color('text_primary')}; font-size: 20px; "
            f"font-weight: 700; background: transparent; letter-spacing: -0.5px;"
        )
        self.new_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {theme_manager.color('accent')};
                color: #FFFFFF;
                border: none;
                border-radius: 15px;
                font-size: 18px;
                font-weight: 600;
            }}
            QPushButton:hover {{
                background-color: {theme_manager.color('accent_hover')};
            }}
        """)
        self.btn_glow.setColor(QColor(theme_manager.color('accent')))
        
        self.search_input.setStyleSheet(f"""
            QLineEdit {{
                background-color: {theme_manager.color('bg_input')};
                border: 1px solid {theme_manager.color('border_subtle')};
                border-radius: 8px;
                padding: 4px 12px;
                font-size: 12px;
                color: {theme_manager.color('text_secondary')};
            }}
            QLineEdit:focus {{
                border-color: {theme_manager.color('accent')};
                background-color: {theme_manager.color('bg_surface')};
            }}
        """)
        
        self.scroll.setStyleSheet("background: transparent; border: none;")
        self.container.setStyleSheet("background: transparent;")
        
        self.settings_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: transparent;
                color: {theme_manager.color('text_muted')};
                text-align: left;
                padding: 8px 4px;
                font-size: 12px;
                font-weight: normal;
                border: none;
                border-radius: 6px;
            }}
            QPushButton:hover {{
                color: {theme_manager.color('text_secondary')};
                background-color: {theme_manager.color('sidebar_hover')};
            }}
        """)

    def add_session(self, session_id: str, name: str, last_msg: str = "", time: str = ""):
        if session_id in self.items:
            return
            
        item = SessionItem(session_id, name, last_msg, time)
        item.clicked.connect(self._on_item_clicked)
        
        self.items[session_id] = item
        # Insert before the stretch
        self.list_layout.insertWidget(self.list_layout.count() - 1, item)
        
        # Select if it's the first one
        if not self.selected_id:
            self.select_session(session_id)

    def select_session(self, session_id: str):
        """Update visual selection state (does NOT emit signal to avoid recursion)."""
        if self.selected_id == session_id:
            return
        if self.selected_id in self.items:
            self.items[self.selected_id].update_selection(False)
            
        self.selected_id = session_id
        if session_id in self.items:
            self.items[session_id].update_selection(True)

    def update_last_message(self, session_id: str, text: str):
        if session_id in self.items:
            self.items[session_id].last_msg_label.setText(text)

    def _on_search(self, text: str):
        """Filter the session list based on search text."""
        text = text.lower()
        for session_id, item in self.items.items():
            visible = text in item.display_name.lower()
            item.setVisible(visible)

    def _on_item_clicked(self, session_id: str):
        """Handle user click — update selection and emit signal."""
        self.select_session(session_id)
        self.session_selected.emit(session_id)
