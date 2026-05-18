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
from .widgets.sidebar import Sidebar
from .theme import COLORS


class MainWindow(QMainWindow):
    """Main application window with session and chat views."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Quicko2")
        self.setMinimumSize(800, 600)  # Wider for sidebar
        self.resize(1000, 720)

        # Session data
        self.sessions = {}  # id -> {info, message_list}
        self.active_session_id = None

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
        """Build the UI layout with sidebar and main content area."""
        central = QWidget()
        self.setCentralWidget(central)

        main_layout = QHBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Sidebar
        self.sidebar = Sidebar()
        self.sidebar.session_selected.connect(self._on_sidebar_session_selected)
        self.sidebar.new_chat_requested.connect(self._on_new_session)
        main_layout.addWidget(self.sidebar)

        # Content area (Stacked)
        self.content_stack = QStackedWidget()
        
        # View 0: Empty state / Welcome
        self.welcome_widget = self._build_welcome_view()
        self.content_stack.addWidget(self.welcome_widget)

        # View 1: Create/Join panel
        self.session_panel = SessionPanel()
        self.session_panel.session_created.connect(self._on_session_created)
        self.session_panel.session_joined.connect(self._on_session_joined)
        self.content_stack.addWidget(self.session_panel)

        # View 2: Active Chat area
        self.chat_view = self._build_chat_view()
        self.content_stack.addWidget(self.chat_view)

        main_layout.addWidget(self.content_stack, 1)

        # Status bar
        self.status = ConnectionStatusBar()
        self.setStatusBar(self.status)

    def _build_welcome_view(self) -> QWidget:
        """Build the default welcome view."""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        
        logo = QLabel("Quicko2")
        logo.setStyleSheet(f"font-size: 48px; font-weight: bold; color: {COLORS['bg_tertiary']};")
        layout.addWidget(logo, alignment=Qt.AlignmentFlag.AlignCenter)
        
        hint = QLabel("Select a chat or start a new one to begin.")
        hint.setStyleSheet(f"color: {COLORS['text_muted']}; font-size: 14px;")
        layout.addWidget(hint, alignment=Qt.AlignmentFlag.AlignCenter)
        
        return widget

    def _build_chat_view(self) -> QWidget:
        """Build the chat view container."""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header bar
        header = QWidget()
        header.setFixedHeight(64)
        header.setStyleSheet(f"background-color: {COLORS['bg_secondary']}; border-bottom: 1px solid {COLORS['border']};")
        header_layout = QHBoxLayout(header)
        header_layout.setContentsMargins(20, 0, 20, 0)

        self.peer_name_label = QLabel("Chat")
        self.peer_name_label.setStyleSheet(f"font-size: 16px; font-weight: 600; color: {COLORS['text_primary']};")
        header_layout.addWidget(self.peer_name_label)

        header_layout.addStretch()

        self.session_code_label = QLabel("")
        self.session_code_label.setStyleSheet(f"font-size: 12px; color: {COLORS['text_muted']}; font-family: monospace; background: {COLORS['bg_tertiary']}; padding: 4px 8px; border-radius: 4px;")
        header_layout.addWidget(self.session_code_label)

        layout.addWidget(header)

        # Message lists stack (one per session)
        self.message_stack = QStackedWidget()
        layout.addWidget(self.message_stack, 1)

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
        self._add_session(session_id, display_name, invite_code)
        self.status.set_status("connected", f"Session active — share code: {invite_code}")

    def _on_session_joined(self, invite_code: str):
        """Handle joining an existing session."""
        import uuid
        session_id = uuid.uuid4().hex[:32]
        display_name = f"Session {invite_code}"
        self._add_session(session_id, display_name, invite_code)
        self.status.set_status("connected", "Joined session")

    def _add_session(self, session_id: str, name: str, code: str):
        """Initialize a new session and switch to it."""
        msg_list = MessageList()
        self.message_stack.addWidget(msg_list)
        
        self.sessions[session_id] = {
            "name": name,
            "code": code,
            "msg_list": msg_list
        }
        
        self.sidebar.add_session(session_id, name)
        self._switch_to_session(session_id)

    def _switch_to_session(self, session_id: str):
        """Switch the UI to show a specific session."""
        if session_id not in self.sessions:
            return

        # If already in the right session AND showing chat view, nothing to do
        if session_id == self.active_session_id and self.content_stack.currentIndex() == 2:
            return

        session = self.sessions[session_id]
        
        # Update UI components
        self.peer_name_label.setText(session["name"])
        self.session_code_label.setText(f"CODE: {session['code']}")
        self.message_stack.setCurrentWidget(session["msg_list"])
        self.content_stack.setCurrentIndex(2)  # Switch to Chat view
        
        # Update sidebar if needed
        if session_id != self.active_session_id:
            self.active_session_id = session_id
            # Temporarily block signals to avoid recursion loops
            self.sidebar.blockSignals(True)
            self.sidebar.select_session(session_id)
            self.sidebar.blockSignals(False)

    def _on_sidebar_session_selected(self, session_id: str):
        self._switch_to_session(session_id)

    def _on_send_message(self, text: str):
        """Handle sending a message in the active session."""
        if not self.active_session_id or not text.strip():
            return
            
        session = self.sessions[self.active_session_id]
        session["msg_list"].add_sent_message(text)
        self.sidebar.update_last_message(self.active_session_id, text)

    def _on_new_session(self):
        """Show the session creation panel."""
        self.content_stack.setCurrentIndex(1)
        self.status.set_status("disconnected", "Creating new session...")
