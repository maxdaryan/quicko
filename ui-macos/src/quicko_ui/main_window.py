"""Main window for Quicko2 — contains session view and chat view."""

from PyQt6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QStackedWidget, QStatusBar, QLabel,
)
from PyQt6.QtCore import Qt, QTimer
from PyQt6.QtGui import QAction

from .widgets.session_panel import SessionPanel
from .widgets.chat_input import ChatInput
from .widgets.message_bubble import MessageList
from .widgets.status_bar import ConnectionStatusBar
from .theme import COLORS


class MainWindow(QMainWindow):
    """Main application window with session and chat views."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Quicko2")
        self.setMinimumSize(480, 640)
        self.resize(520, 720)

        # Center on screen
        screen = self.screen()
        if screen:
            geo = screen.availableGeometry()
            self.move(
                (geo.width() - self.width()) // 2,
                (geo.height() - self.height()) // 2,
            )

        self._setup_ui()
        self._setup_menu()

    def _setup_ui(self):
        """Build the UI layout."""
        central = QWidget()
        self.setCentralWidget(central)

        layout = QVBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Stacked widget for session/chat views
        self.stack = QStackedWidget()

        # View 0: Session panel (create/join)
        self.session_panel = SessionPanel()
        self.session_panel.session_created.connect(self._on_session_created)
        self.session_panel.session_joined.connect(self._on_session_joined)
        self.stack.addWidget(self.session_panel)

        # View 1: Chat view
        self.chat_widget = self._build_chat_view()
        self.stack.addWidget(self.chat_widget)

        layout.addWidget(self.stack)

        # Status bar
        self.status = ConnectionStatusBar()
        self.setStatusBar(self.status)

    def _build_chat_view(self) -> QWidget:
        """Build the chat view with message list and input."""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header bar
        header = QWidget()
        header.setFixedHeight(56)
        header.setStyleSheet(f"background-color: {COLORS['bg_secondary']}; border-bottom: 1px solid {COLORS['border']};")
        header_layout = QHBoxLayout(header)
        header_layout.setContentsMargins(16, 0, 16, 0)

        self.peer_name_label = QLabel("Waiting for peer...")
        self.peer_name_label.setStyleSheet(f"font-size: 16px; font-weight: 600; color: {COLORS['text_primary']};")
        header_layout.addWidget(self.peer_name_label)

        header_layout.addStretch()

        self.session_code_label = QLabel("")
        self.session_code_label.setStyleSheet(f"font-size: 12px; color: {COLORS['text_muted']}; font-family: monospace;")
        header_layout.addWidget(self.session_code_label)

        layout.addWidget(header)

        # Message list
        self.message_list = MessageList()
        layout.addWidget(self.message_list, 1)

        # Chat input
        self.chat_input = ChatInput()
        self.chat_input.message_submitted.connect(self._on_send_message)
        layout.addWidget(self.chat_input)

        return widget

    def _setup_menu(self):
        """Set up the macOS menu bar."""
        menubar = self.menuBar()

        # File menu
        file_menu = menubar.addMenu("File")

        new_session = QAction("New Session", self)
        new_session.setShortcut("Ctrl+N")
        new_session.triggered.connect(self._on_new_session)
        file_menu.addAction(new_session)

        file_menu.addSeparator()

        quit_action = QAction("Quit", self)
        quit_action.setShortcut("Ctrl+Q")
        quit_action.triggered.connect(self.close)
        file_menu.addAction(quit_action)

    def _on_session_created(self, session_id: str, display_name: str, invite_code: str):
        """Handle session creation."""
        self.session_code_label.setText(f"Code: {invite_code}")
        self.peer_name_label.setText("Waiting for peer...")
        self.stack.setCurrentIndex(1)  # Switch to chat view
        self.status.set_status("connected", f"Session active — share code: {invite_code}")

    def _on_session_joined(self, invite_code: str):
        """Handle joining an existing session."""
        self.session_code_label.setText(f"Code: {invite_code}")
        self.peer_name_label.setText("Connected")
        self.stack.setCurrentIndex(1)
        self.status.set_status("connected", "Joined session")

    def _on_send_message(self, text: str):
        """Handle sending a message."""
        if text.strip():
            self.message_list.add_sent_message(text)

    def _on_new_session(self):
        """Reset to session panel."""
        self.stack.setCurrentIndex(0)
        self.status.set_status("disconnected", "No active session")
