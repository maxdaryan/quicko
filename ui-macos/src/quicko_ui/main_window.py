"""Main window for Quicko2 — contains session view and chat view."""

from PyQt6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QStackedWidget, QStatusBar, QLabel, QGraphicsDropShadowEffect,
    QGraphicsOpacityEffect, QSizePolicy,
)
from PyQt6.QtCore import Qt, QTimer, QPropertyAnimation, QEasingCurve, QSize
from PyQt6.QtGui import QAction, QColor, QPainter, QLinearGradient, QPen, QFont

from .widgets.session_panel import SessionPanel
from .widgets.chat_input import ChatInput
from .widgets.message_bubble import MessageList
from .widgets.status_bar import ConnectionStatusBar
from .widgets.sidebar import Sidebar
from .theme import theme_manager, MONO_FONT_STACK


class GlowDot(QWidget):
    """A small animated glowing dot for the welcome screen."""
    
    def __init__(self, color: str = "#7C6CF7", size: int = 8, parent=None):
        super().__init__(parent)
        self.dot_color = QColor(color)
        self.dot_size = size
        self.setFixedSize(size + 4, size + 4)
        
        # Pulse animation via opacity
        self._opacity_effect = QGraphicsOpacityEffect(self)
        self._opacity_effect.setOpacity(0.6)
        self.setGraphicsEffect(self._opacity_effect)
        
        self._pulse = QPropertyAnimation(self._opacity_effect, b"opacity")
        self._pulse.setDuration(2000)
        self._pulse.setStartValue(0.3)
        self._pulse.setEndValue(1.0)
        self._pulse.setEasingCurve(QEasingCurve.Type.InOutSine)
        self._pulse.setLoopCount(-1)  # Infinite
        # Make it ping-pong by reversing direction each cycle
        self._pulse.finished.connect(lambda: None)
        self._pulse.start()
    
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)
        painter.setPen(Qt.PenStyle.NoPen)
        painter.setBrush(self.dot_color)
        x = (self.width() - self.dot_size) // 2
        y = (self.height() - self.dot_size) // 2
        painter.drawEllipse(x, y, self.dot_size, self.dot_size)


class MainWindow(QMainWindow):
    """Main application window with session and chat views."""

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Quicko2")
        self.setMinimumSize(900, 640)
        self.resize(1060, 760)

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
        self.update_theme()
        theme_manager.theme_changed.connect(self.update_theme)

    def update_theme(self):
        self.welcome_widget.setStyleSheet(f"background-color: {theme_manager.color('bg_primary')};")
        self.cat_label.setStyleSheet(
            f"font-family: {MONO_FONT_STACK}; "
            f"font-size: 16px; "
            f"color: {theme_manager.color('accent')}; "
            f"line-height: 1.4; "
            f"letter-spacing: 0px; "
            f"background: transparent;"
        )
        self.logo_label.setStyleSheet(
            f"font-size: 56px; "
            f"font-weight: 800; "
            f"color: {theme_manager.color('accent')}; "
            f"background: transparent; "
            f"letter-spacing: -1px;"
        )
        self.logo_glow.setColor(QColor(theme_manager.color('accent')))
        
        self.tagline.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 14px; "
            f"font-weight: 400; "
            f"letter-spacing: 3px; "
            f"text-transform: uppercase; "
            f"background: transparent;"
        )
        
        self.hint.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 13px; "
            f"background: {theme_manager.color('accent_subtle')}; "
            f"padding: 10px 24px; "
            f"border-radius: 20px; "
            f"border: 1px solid {theme_manager.color('border')};"
        )
        
        self.version_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 11px; "
            f"background: transparent; "
            f"padding-bottom: 12px;"
        )
        
        self.chat_header.setStyleSheet(
            f"background: qlineargradient(x1:0, y1:0, x2:1, y2:0, "
            f"stop:0 {theme_manager.color('bg_secondary')}, stop:1 {theme_manager.color('bg_surface')}); "
            f"border-bottom: 1px solid {theme_manager.color('border')};"
        )
        
        self.peer_name_label.setStyleSheet(
            f"font-size: 16px; font-weight: 600; "
            f"color: {theme_manager.color('text_primary')}; background: transparent;"
        )
        
        self.session_code_label.setStyleSheet(
            f"font-size: 11px; "
            f"color: {theme_manager.color('accent')}; "
            f"font-family: {MONO_FONT_STACK}; "
            f"background: {theme_manager.color('accent_subtle')}; "
            f"padding: 4px 12px; "
            f"border-radius: 12px; "
            f"border: 1px solid {theme_manager.color('border')};"
        )

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
        """Build the default welcome view with centered logo and animated accents."""
        widget = QWidget()
        widget.setStyleSheet(f"background-color: {theme_manager.color('bg_primary')};")
        layout = QVBoxLayout(widget)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(16)

        # Add top spacer
        layout.addStretch(2)

        # Owl mascot — properly aligned monospace ASCII art
        # NOTE: letter-spacing must be 0 for monospace art to align correctly.
        # Any positive letter-spacing shifts each character slightly, causing
        # rows of different lengths to visually diverge.
        owl_art = (
            "  ╔═══════╗  \n"
            "  ║ O   O ║  \n"
            "  ║   v   ║  \n"
            "  ║ ───── ║  \n"
            "  ╚══╗ ╔══╝  \n"
            "     ║ ║     \n"
            "    ─╝ ╚─    "
        )
        self.cat_label = QLabel(owl_art)
        self.cat_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.cat_label.setStyleSheet(
            f"font-family: {MONO_FONT_STACK}; "
            f"font-size: 16px; "
            f"color: {theme_manager.color('accent')}; "
            f"line-height: 1.4; "
            f"letter-spacing: 0px; "
            f"background: transparent;"
        )
        layout.addWidget(self.cat_label, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addSpacing(8)

        # App name — large gradient-like styled text
        self.logo_label = QLabel("Quicko")
        self.logo_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.logo_label.setStyleSheet(
            f"font-size: 56px; "
            f"font-weight: 800; "
            f"color: {theme_manager.color('accent')}; "
            f"background: transparent; "
            f"letter-spacing: -1px;"
        )
        # Add glow shadow effect
        self.logo_glow = QGraphicsDropShadowEffect()
        self.logo_glow.setBlurRadius(40)
        self.logo_glow.setColor(QColor(theme_manager.color('accent')))
        self.logo_glow.setOffset(0, 0)
        self.logo_label.setGraphicsEffect(self.logo_glow)
        layout.addWidget(self.logo_label, alignment=Qt.AlignmentFlag.AlignCenter)

        # Tagline
        self.tagline = QLabel("Ephemeral · Encrypted · Instant")
        self.tagline.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.tagline.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 14px; "
            f"font-weight: 400; "
            f"letter-spacing: 3px; "
            f"text-transform: uppercase; "
            f"background: transparent;"
        )
        layout.addWidget(self.tagline, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addSpacing(32)

        # Action hint with subtle styling
        self.hint = QLabel("Press  ⌘N  to start a new conversation")
        self.hint.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.hint.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 13px; "
            f"background: {theme_manager.color('accent_subtle')}; "
            f"padding: 10px 24px; "
            f"border-radius: 20px; "
            f"border: 1px solid {theme_manager.color('border')};"
        )
        layout.addWidget(self.hint, alignment=Qt.AlignmentFlag.AlignCenter)

        # Bottom spacer
        layout.addStretch(3)

        # Subtle version label
        self.version_label = QLabel("v0.1.0 — Pre-release")
        self.version_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.version_label.setStyleSheet(
            f"color: {theme_manager.color('text_muted')}; "
            f"font-size: 11px; "
            f"background: transparent; "
            f"padding-bottom: 12px;"
        )
        layout.addWidget(self.version_label, alignment=Qt.AlignmentFlag.AlignCenter)

        return widget

    def _build_chat_view(self) -> QWidget:
        """Build the chat view container."""
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header bar with gradient accent
        self.chat_header = QWidget()
        self.chat_header.setFixedHeight(60)
        header_layout = QHBoxLayout(self.chat_header)
        header_layout.setContentsMargins(24, 0, 24, 0)

        self.peer_name_label = QLabel("Chat")
        header_layout.addWidget(self.peer_name_label)

        header_layout.addStretch()

        self.session_code_label = QLabel("")
        header_layout.addWidget(self.session_code_label)

        layout.addWidget(self.chat_header)

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
