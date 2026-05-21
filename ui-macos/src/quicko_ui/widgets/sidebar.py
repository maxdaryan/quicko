"""Sidebar widget for Quicko2 — lists active and previous sessions."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
    QScrollArea, QPushButton, QFrame, QLineEdit,
)
from PyQt6.QtCore import Qt, pyqtSignal, QSize
from PyQt6.QtGui import QFont, QColor

from ..theme import theme_manager


class SessionItem(QFrame):
    """A single session entry in the sidebar."""
    
    clicked = pyqtSignal(str)

    def __init__(self, session_id: str, display_name: str, last_msg: str = "", timestamp: str = ""):
        super().__init__()
        self.setObjectName("SessionItem")
        self.session_id = session_id
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setProperty("selected", "false")
        
        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(12)

        # Avatar
        self.avatar = QLabel(display_name[0].upper() if display_name else "?")
        self.avatar.setFixedSize(40, 40)
        self.avatar.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.avatar.setStyleSheet(f"""
            background-color: {theme_manager.color('bg_tertiary')};
            color: {theme_manager.color('accent')};
            border-radius: 20px;
            font-weight: 800;
            font-size: 16px;
        """)
        layout.addWidget(self.avatar)

        # Info
        info_layout = QVBoxLayout()
        info_layout.setSpacing(2)
        
        self.name_label = QLabel(display_name)
        self.name_label.setStyleSheet("font-weight: 600; font-size: 13px; color: palette(text);")
        info_layout.addWidget(self.name_label)

        self.last_msg_label = QLabel(last_msg or "No messages yet")
        self.last_msg_label.setStyleSheet(f"color: {theme_manager.color('text_secondary')}; font-size: 12px;")
        self.last_msg_label.setMinimumWidth(120)
        info_layout.addWidget(self.last_msg_label)

        layout.addLayout(info_layout)
        layout.addStretch()

        if timestamp:
            self.time_label = QLabel(timestamp)
            self.time_label.setStyleSheet(f"color: {theme_manager.color('text_muted')}; font-size: 10px;")
            layout.addWidget(self.time_label, alignment=Qt.AlignmentFlag.AlignTop)

    def update_selection(self, selected: bool):
        self.setProperty("selected", str(selected).lower())
        self.style().unpolish(self)
        self.style().polish(self)

    def mousePressEvent(self, event):
        self.clicked.emit(self.session_id)
        super().mousePressEvent(event)


class Sidebar(QWidget):
    """Sidebar containing the session list and 'New Chat' button."""
    
    session_selected = pyqtSignal(str)
    new_chat_requested = pyqtSignal()

    def __init__(self):
        super().__init__()
        self.setObjectName("Sidebar")
        self.setFixedWidth(280)
        
        self.items = {}
        self.selected_id = None
        
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header
        header_widget = QWidget()
        header_layout = QHBoxLayout(header_widget)
        header_layout.setContentsMargins(20, 20, 20, 10)
        
        title = QLabel("Chats")
        title.setStyleSheet("font-size: 22px; font-weight: 800; letter-spacing: -0.5px;")
        header_layout.addWidget(title)
        header_layout.addStretch()
        
        self.new_btn = QPushButton("+")
        self.new_btn.setFixedSize(32, 32)
        self.new_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.new_btn.clicked.connect(self.new_chat_requested.emit)
        header_layout.addWidget(self.new_btn)
        
        layout.addWidget(header_widget)

        # Search
        search_container = QWidget()
        search_layout = QVBoxLayout(search_container)
        search_layout.setContentsMargins(16, 0, 16, 12)
        
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search")
        self.search_input.setFixedHeight(36)
        search_layout.addWidget(self.search_input)
        layout.addWidget(search_container)

        # Scroll Area
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        
        self.container = QWidget()
        self.list_layout = QVBoxLayout(self.container)
        self.list_layout.setContentsMargins(0, 0, 0, 0)
        self.list_layout.setSpacing(0)
        self.list_layout.addStretch()
        
        self.scroll.setWidget(self.container)
        layout.addWidget(self.scroll, 1)

        # Settings
        self.settings_btn = QPushButton("≡  Settings")
        self.settings_btn.setProperty("secondary", "true")
        self.settings_btn.setFlat(True)
        self.settings_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.settings_btn.clicked.connect(theme_manager.toggle_theme)
        
        settings_layout = QVBoxLayout()
        settings_layout.setContentsMargins(16, 12, 16, 16)
        settings_layout.addWidget(self.settings_btn)
        layout.addLayout(settings_layout)

    def add_session(self, session_id: str, name: str, last_msg: str = "", time: str = ""):
        if session_id in self.items:
            return
        item = SessionItem(session_id, name, last_msg, time)
        item.clicked.connect(self._on_item_clicked)
        self.items[session_id] = item
        self.list_layout.insertWidget(self.list_layout.count() - 1, item)
        if not self.selected_id:
            self.select_session(session_id)

    def select_session(self, session_id: str):
        if self.selected_id in self.items:
            self.items[self.selected_id].update_selection(False)
        self.selected_id = session_id
        if session_id in self.items:
            self.items[session_id].update_selection(True)

    def update_last_message(self, session_id: str, text: str):
        if session_id in self.items:
            self.items[session_id].last_msg_label.setText(text)

    def _on_item_clicked(self, session_id: str):
        self.select_session(session_id)
        self.session_selected.emit(session_id)

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
