"""Sidebar widget for Quicko2 — lists active and previous sessions."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
    QScrollArea, QPushButton, QFrame, QLineEdit,
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QFont

from ..theme import COLORS


class SessionItem(QFrame):
    """A single session entry in the sidebar."""
    
    clicked = pyqtSignal(str)  # session_id

    def __init__(self, session_id: str, display_name: str, last_msg: str = "", timestamp: str = "", is_active: bool = False):
        super().__init__()
        self.session_id = session_id
        self.display_name = display_name
        self.is_active = is_active
        self._setup_ui(display_name, last_msg, timestamp)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.update_selection(False)

    def _setup_ui(self, name, last_msg, time):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(12)

        # Avatar circle
        avatar = QLabel(name[0].upper() if name else "?")
        avatar.setFixedSize(36, 36)
        avatar.setAlignment(Qt.AlignmentFlag.AlignCenter)
        avatar.setStyleSheet(f"""
            QLabel {{
                background-color: {COLORS['bg_tertiary']};
                color: {COLORS['accent']};
                border-radius: 18px;
                font-weight: bold;
                font-size: 16px;
            }}
        """)
        layout.addWidget(avatar)

        # Info
        info_layout = QVBoxLayout()
        info_layout.setSpacing(2)
        
        name_label = QLabel(name)
        name_label.setStyleSheet(f"color: {COLORS['text_primary']}; font-weight: 600; font-size: 13px; background: transparent;")
        info_layout.addWidget(name_label)

        self.last_msg_label = QLabel(last_msg or "No messages yet")
        self.last_msg_label.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 12px; background: transparent;")
        self.last_msg_label.setFixedWidth(140)
        info_layout.addWidget(self.last_msg_label)

        layout.addLayout(info_layout)
        layout.addStretch()

        # Time
        if time:
            time_label = QLabel(time)
            time_label.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 10px; background: transparent;")
            layout.addWidget(time_label, alignment=Qt.AlignmentFlag.AlignTop)

    def update_selection(self, selected: bool):
        bg = COLORS['bg_hover'] if selected else "transparent"
        self.setStyleSheet(f"""
            QFrame {{
                background-color: {bg};
                border-radius: 8px;
            }}
            QFrame:hover {{
                background-color: {COLORS['bg_secondary'] if not selected else COLORS['bg_hover']};
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
        self.setFixedWidth(260)
        self.setStyleSheet(f"background-color: {COLORS['bg_secondary']}; border-right: 1px solid {COLORS['border']};")
        
        self.items = {}  # session_id -> SessionItem
        self.selected_id = None
        
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 16, 12, 16)
        layout.setSpacing(16)

        # Header
        header = QHBoxLayout()
        title = QLabel("Chats")
        title.setStyleSheet(f"color: {COLORS['text_primary']}; font-size: 18px; font-weight: bold; background: transparent;")
        header.addWidget(title)
        
        new_btn = QPushButton("+")
        new_btn.setFixedSize(28, 28)
        new_btn.setToolTip("New Chat")
        new_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        new_btn.clicked.connect(self.new_chat_requested.emit)
        header.addWidget(new_btn)
        
        layout.addLayout(header)

        # Search Bar
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search chats...")
        self.search_input.setFixedHeight(32)
        self.search_input.setStyleSheet(f"""
            QLineEdit {{
                background-color: {COLORS['bg_input']};
                border: none;
                border-radius: 6px;
                padding: 4px 10px;
                font-size: 12px;
            }}
        """)
        self.search_input.textChanged.connect(self._on_search)
        layout.addWidget(self.search_input)

        # Scroll Area for sessions
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setStyleSheet("background: transparent; border: none;")
        
        self.container = QWidget()
        self.container.setStyleSheet("background: transparent;")
        self.list_layout = QVBoxLayout(self.container)
        self.list_layout.setContentsMargins(0, 0, 0, 0)
        self.list_layout.setSpacing(4)
        self.list_layout.addStretch()
        
        self.scroll.setWidget(self.container)
        layout.addWidget(self.scroll)

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
