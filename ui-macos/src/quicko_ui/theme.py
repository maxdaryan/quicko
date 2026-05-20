"""Dark theme for Quicko2 macOS UI.

Provides a sleek, macOS-native-feeling dark theme using Qt stylesheets.
Uses system-available fonts with proper fallbacks to avoid font loading crashes.
"""

from PyQt6.QtWidgets import QApplication
from PyQt6.QtGui import QPalette, QColor, QFontDatabase, QFont
from PyQt6.QtCore import Qt

# Color palette — refined with richer tones
COLORS = {
    "bg_primary": "#0A0A0F",
    "bg_secondary": "#12121A",
    "bg_tertiary": "#1C1C2B",
    "bg_hover": "#252538",
    "bg_input": "#16161F",
    "bg_surface": "#1A1A28",
    "text_primary": "#EAEAF0",
    "text_secondary": "#9898B0",
    "text_muted": "#5A5A72",
    "accent": "#7C6CF7",
    "accent_hover": "#8E7FFF",
    "accent_pressed": "#6A5AE0",
    "accent_glow": "rgba(124, 108, 247, 0.15)",
    "accent_subtle": "rgba(124, 108, 247, 0.08)",
    "success": "#00D68F",
    "warning": "#FFB800",
    "error": "#FF3D71",
    "border": "#22223A",
    "border_focus": "#7C6CF7",
    "border_subtle": "#1A1A30",
    "message_sent": "#2D2458",
    "message_received": "#1A1A2E",
    "scrollbar": "#2A2A42",
    "scrollbar_hover": "#3A3A58",
    "gradient_start": "#7C6CF7",
    "gradient_end": "#A78BFA",
    "sidebar_bg": "#0E0E16",
    "sidebar_hover": "#1A1A2A",
    "avatar_bg": "#252540",
}


LIGHT_COLORS = {
    "bg_primary": "#FFFFFF",
    "bg_secondary": "#F3F4F6",
    "bg_tertiary": "#E5E7EB",
    "bg_hover": "#E5E7EB",
    "bg_input": "#FFFFFF",
    "bg_surface": "#F9FAFB",
    "text_primary": "#111827",
    "text_secondary": "#4B5563",
    "text_muted": "#6B7280",
    "accent": "#6366F1",
    "accent_hover": "#4F46E5",
    "accent_pressed": "#4338CA",
    "accent_glow": "rgba(99, 102, 241, 0.15)",
    "accent_subtle": "rgba(99, 102, 241, 0.08)",
    "success": "#10B981",
    "warning": "#F59E0B",
    "error": "#EF4444",
    "border": "#D1D5DB",
    "border_focus": "#6366F1",
    "border_subtle": "#E5E7EB",
    "message_sent": "#E0E7FF",
    "message_received": "#F3F4F6",
    "scrollbar": "#D1D5DB",
    "scrollbar_hover": "#9CA3AF",
    "gradient_start": "#6366F1",
    "gradient_end": "#8B5CF6",
    "sidebar_bg": "#F9FAFB",
    "sidebar_hover": "#F3F4F6",
    "avatar_bg": "#E5E7EB",
}

from PyQt6.QtCore import QObject, pyqtSignal

class _ThemeManager(QObject):
    theme_changed = pyqtSignal()
    
    def __init__(self):
        super().__init__()
        self.is_dark = True
        self.COLORS = COLORS  # Defaults to dark initially
        
    def toggle_theme(self):
        self.is_dark = not self.is_dark
        self.COLORS = COLORS if self.is_dark else LIGHT_COLORS
        self.theme_changed.emit()
        
    def color(self, key):
        return self.COLORS[key]

theme_manager = _ThemeManager()


# Preferred font stack — uses only real font family names that Qt can resolve
FONT_STACK = "'Helvetica Neue', Arial"
MONO_FONT_STACK = "'Menlo', 'Monaco', 'Consolas'"


def _resolve_font(app: QApplication) -> str:
    """Find the best available font family from our preferences."""
    families = QFontDatabase.families()
    
    # Priority list — avoid alias names (e.g. "SF Pro Display") that Qt can't
    # resolve directly; they trigger a slow 80ms alias-population pass and a
    # console warning.  Use the private internal name or concrete fallbacks.
    preferred = [
        ".AppleSystemUIFont",   # macOS native system font (Qt resolves this cleanly)
        "SF Pro Text",          # Available on macOS 11+ as a real family
        "Helvetica Neue",
        "Segoe UI",
        "Inter",
        "Roboto",
        "Arial",
    ]
    
    for font_name in preferred:
        if font_name in families:
            return font_name
    
    # Ultimate fallback
    return "Helvetica"


def apply_theme(app: QApplication = None):
    """Apply the Quicko2 theme to the application."""
    if app is None:
        app = QApplication.instance()
        if app is None:
            return

    # Resolve a font that actually exists on this system
    resolved_font = _resolve_font(app)    
    # Set default font
    font = QFont(resolved_font, 13)
    font.setStyleStrategy(QFont.StyleStrategy.PreferAntialias)
    app.setFont(font)
    
    # Set palette
    palette = QPalette()
    palette.setColor(QPalette.ColorRole.Window, QColor(theme_manager.color("bg_primary")))
    palette.setColor(QPalette.ColorRole.WindowText, QColor(theme_manager.color("text_primary")))
    palette.setColor(QPalette.ColorRole.Base, QColor(theme_manager.color("bg_secondary")))
    palette.setColor(QPalette.ColorRole.AlternateBase, QColor(theme_manager.color("bg_tertiary")))
    palette.setColor(QPalette.ColorRole.Text, QColor(theme_manager.color("text_primary")))
    palette.setColor(QPalette.ColorRole.Button, QColor(theme_manager.color("bg_tertiary")))
    palette.setColor(QPalette.ColorRole.ButtonText, QColor(theme_manager.color("text_primary")))
    palette.setColor(QPalette.ColorRole.Highlight, QColor(theme_manager.color("accent")))
    palette.setColor(QPalette.ColorRole.HighlightedText, QColor("#FFFFFF"))
    palette.setColor(QPalette.ColorRole.PlaceholderText, QColor(theme_manager.color("text_muted")))
    app.setPalette(palette)

    # Build global stylesheet with the resolved font
    stylesheet = _build_stylesheet(resolved_font)
    app.setStyleSheet(stylesheet)


def _build_stylesheet(font_family: str) -> str:
    """Build the global stylesheet using a verified font family."""
    return f"""
/* ===== Global ===== */
QWidget {{
    background-color: {theme_manager.color("bg_primary")};
    color: {theme_manager.color("text_primary")};
    font-family: "{font_family}", {FONT_STACK};
    selection-background-color: {theme_manager.color("accent")};
    selection-color: #FFFFFF;
}}

/* ===== Scrollbar ===== */
QScrollBar:vertical {{
    background: transparent;
    width: 6px;
    margin: 4px 2px;
}}
QScrollBar::handle:vertical {{
    background: {theme_manager.color("scrollbar")};
    border-radius: 3px;
    min-height: 30px;
}}
QScrollBar::handle:vertical:hover {{
    background: {theme_manager.color("scrollbar_hover")};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
    height: 0;
}}
QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{
    background: transparent;
}}

/* ===== Input Fields ===== */
QLineEdit, QTextEdit, QPlainTextEdit {{
    background-color: {theme_manager.color("bg_input")};
    border: 1px solid {theme_manager.color("border")};
    border-radius: 10px;
    padding: 8px 14px;
    color: {theme_manager.color("text_primary")};
    font-size: 14px;
}}
QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {{
    border-color: {theme_manager.color("border_focus")};
}}

/* ===== Buttons ===== */
QPushButton {{
    background-color: {theme_manager.color("accent")};
    color: #FFFFFF;
    border: none;
    border-radius: 10px;
    padding: 10px 22px;
    font-size: 14px;
    font-weight: 600;
}}
QPushButton:hover {{
    background-color: {theme_manager.color("accent_hover")};
}}
QPushButton:pressed {{
    background-color: {theme_manager.color("accent_pressed")};
}}
QPushButton:disabled {{
    background-color: {theme_manager.color("bg_tertiary")};
    color: {theme_manager.color("text_muted")};
}}

/* ===== Secondary Button ===== */
QPushButton[cssClass="secondary"] {{
    background-color: {theme_manager.color("bg_tertiary")};
    color: {theme_manager.color("text_primary")};
    border: 1px solid {theme_manager.color("border")};
}}
QPushButton[cssClass="secondary"]:hover {{
    background-color: {theme_manager.color("bg_hover")};
    border-color: {theme_manager.color("accent")};
}}

/* ===== Labels ===== */
QLabel {{
    background-color: transparent;
}}
QLabel[cssClass="title"] {{
    font-size: 24px;
    font-weight: 700;
    color: {theme_manager.color("text_primary")};
}}
QLabel[cssClass="subtitle"] {{
    font-size: 14px;
    color: {theme_manager.color("text_secondary")};
}}
QLabel[cssClass="muted"] {{
    font-size: 12px;
    color: {theme_manager.color("text_muted")};
}}

/* ===== Status Bar ===== */
QStatusBar {{
    background-color: {theme_manager.color("bg_secondary")};
    color: {theme_manager.color("text_muted")};
    font-size: 12px;
    border-top: 1px solid {theme_manager.color("border")};
}}

/* ===== Tool Tips ===== */
QToolTip {{
    background-color: {theme_manager.color("bg_tertiary")};
    color: {theme_manager.color("text_primary")};
    border: 1px solid {theme_manager.color("border")};
    border-radius: 6px;
    padding: 6px 10px;
}}
"""


# Connect theme changes
theme_manager.theme_changed.connect(apply_theme)

