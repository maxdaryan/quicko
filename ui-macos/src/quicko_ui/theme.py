"""Quicko2 Theme System.

Provides a unified styling engine with support for both Dark and Light modes.
Uses dynamic properties to avoid frequent setStyleSheet calls which cause jank.
"""

from PyQt6.QtWidgets import QApplication
from PyQt6.QtGui import QPalette, QColor, QFontDatabase, QFont
from PyQt6.QtCore import Qt, QObject, pyqtSignal

# High-fidelity color palette
COLORS = {
    # Backgrounds
    "bg_primary": "#0D0D12",
    "bg_secondary": "#16161E",
    "bg_tertiary": "#1F1F2E",
    "bg_hover": "#2A2A3D",
    "bg_input": "#1A1A26",
    "bg_surface": "#1E1E2C",
    "sidebar_bg": "#09090D",
    "sidebar_hover": "#1A1A26",
    
    # Text
    "text_primary": "#F2F2F7",
    "text_secondary": "#A9A9B2",
    "text_muted": "#636366",
    "text_on_accent": "#FFFFFF",
    
    # Accents
    "accent": "#5E5CE6",  # iOS System Blue (Indigo)
    "accent_hover": "#7D7AFF",
    "accent_pressed": "#4544C4",
    "accent_glow": "rgba(94, 92, 230, 0.3)",
    "accent_subtle": "rgba(94, 92, 230, 0.12)",
    
    # States
    "success": "#32D74B",
    "warning": "#FF9F0A",
    "error": "#FF453A",
    "border": "#2C2C2E",
    "border_focus": "#5E5CE6",
    "border_subtle": "#1C1C1E",
    
    # Messaging
    "message_sent": "#5E5CE6",
    "message_received": "#2C2C2E",
    "scrollbar": "rgba(255, 255, 255, 0.15)",
    "scrollbar_hover": "rgba(255, 255, 255, 0.25)",
}

LIGHT_COLORS = {
    "bg_primary": "#FFFFFF",
    "bg_secondary": "#F2F2F7",
    "bg_tertiary": "#E5E5EA",
    "bg_hover": "#D1D1D6",
    "bg_input": "#FFFFFF",
    "bg_surface": "#F9F9F9",
    "sidebar_bg": "#F2F2F7",
    "sidebar_hover": "#E5E5EA",
    
    "text_primary": "#000000",
    "text_secondary": "#3C3C43",
    "text_muted": "#8E8E93",
    "text_on_accent": "#FFFFFF",
    
    "accent": "#007AFF",
    "accent_hover": "#3498FF",
    "accent_pressed": "#0051A8",
    "accent_glow": "rgba(0, 122, 255, 0.2)",
    "accent_subtle": "rgba(0, 122, 255, 0.1)",
    
    "success": "#34C759",
    "warning": "#FF9500",
    "error": "#FF3B30",
    "border": "#C6C6C8",
    "border_focus": "#007AFF",
    "border_subtle": "#E5E5EA",
    
    "message_sent": "#007AFF",
    "message_received": "#E5E5EA",
    "scrollbar": "rgba(0, 0, 0, 0.15)",
    "scrollbar_hover": "rgba(0, 0, 0, 0.25)",
}

class _ThemeManager(QObject):
    theme_changed = pyqtSignal()
    
    def __init__(self):
        super().__init__()
        self.is_dark = True
        self.COLORS = COLORS
        
    def toggle_theme(self):
        self.is_dark = not self.is_dark
        self.COLORS = COLORS if self.is_dark else LIGHT_COLORS
        self.theme_changed.emit()
        
    def color(self, key):
        return self.COLORS.get(key, "#FF00FF")

theme_manager = _ThemeManager()

FONT_STACK = ".AppleSystemUIFont, 'SF Pro Text', 'Helvetica Neue', Arial"
MONO_FONT_STACK = "'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace"

def apply_theme(app: QApplication = None):
    if app is None:
        app = QApplication.instance()
    if app is None:
        return

    # Set Palette for native dialogs
    palette = QPalette()
    bg = QColor(theme_manager.color("bg_primary"))
    text = QColor(theme_manager.color("text_primary"))
    accent = QColor(theme_manager.color("accent"))
    
    palette.setColor(QPalette.ColorRole.Window, bg)
    palette.setColor(QPalette.ColorRole.WindowText, text)
    palette.setColor(QPalette.ColorRole.Base, QColor(theme_manager.color("bg_secondary")))
    palette.setColor(QPalette.ColorRole.Text, text)
    palette.setColor(QPalette.ColorRole.Button, QColor(theme_manager.color("bg_tertiary")))
    palette.setColor(QPalette.ColorRole.ButtonText, text)
    palette.setColor(QPalette.ColorRole.Highlight, accent)
    palette.setColor(QPalette.ColorRole.HighlightedText, QColor("#FFFFFF"))
    app.setPalette(palette)

    # Global Stylesheet
    app.setStyleSheet(_build_stylesheet())

def _build_stylesheet() -> str:
    c = theme_manager.color
    return f"""
* {{
    font-family: {FONT_STACK};
    outline: none;
}}

QWidget {{
    background-color: transparent;
    color: {c("text_primary")};
}}

QMainWindow, QDialog {{
    background-color: {c("bg_primary")};
}}

/* ===== Scrollbar ===== */
QScrollBar:vertical {{
    background: transparent;
    width: 8px;
    margin: 2px;
}}
QScrollBar::handle:vertical {{
    background: {c("scrollbar")};
    border-radius: 4px;
    min-height: 40px;
}}
QScrollBar::handle:vertical:hover {{
    background: {c("scrollbar_hover")};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
    height: 0;
}}

/* ===== Buttons ===== */
QPushButton {{
    background-color: {c("accent")};
    color: {c("text_on_accent")};
    border: none;
    border-radius: 8px;
    padding: 8px 16px;
    font-weight: 600;
}}
QPushButton:hover {{
    background-color: {c("accent_hover")};
}}
QPushButton:pressed {{
    background-color: {c("accent_pressed")};
}}

QPushButton[secondary="true"] {{
    background-color: {c("bg_tertiary")};
    color: {c("text_primary")};
}}
QPushButton[secondary="true"]:hover {{
    background-color: {c("bg_hover")};
}}

/* ===== Input ===== */
QLineEdit, QTextEdit, QPlainTextEdit {{
    background-color: {c("bg_input")};
    border: 1px solid {c("border")};
    border-radius: 10px;
    padding: 8px 12px;
    selection-background-color: {c("accent")};
}}
QLineEdit:focus {{
    border: 1px solid {c("accent")};
}}

/* ===== Sidebar ===== */
#Sidebar {{
    background-color: {c("sidebar_bg")};
    border-right: 1px solid {c("border")};
}}

#SessionItem {{
    border-radius: 8px;
    margin: 2px 8px;
}}
#SessionItem[selected="true"] {{
    background-color: {c("bg_hover")};
}}
#SessionItem:hover {{
    background-color: {c("sidebar_hover")};
}}

/* ===== Message Bubble ===== */
#MessageBubble {{
    border-radius: 16px;
    padding: 10px;
    margin: 4px;
}}
#MessageBubble[sent="true"] {{
    background-color: {c("message_sent")};
    border-bottom-right-radius: 4px;
}}
#MessageBubble[sent="false"] {{
    background-color: {c("message_received")};
    border-bottom-left-radius: 4px;
}}

#MessageText {{
    color: {c("text_primary")};
    font-size: 14px;
}}
#MessageTime {{
    color: {c("text_muted")};
    font-size: 10px;
}}
#MessageTime[sent="true"] {{
    color: rgba(255, 255, 255, 0.7);
}}

/* ===== Welcome Screen ===== */
#WelcomeLogo {{
    font-size: 64px;
    font-weight: 800;
    color: {c("accent")};
    letter-spacing: -2px;
}}
#WelcomeTagline {{
    color: {c("text_secondary")};
    font-size: 16px;
    letter-spacing: 0.5px;
}}
"""


# Connect theme changes
theme_manager.theme_changed.connect(apply_theme)

