"""Dark theme for Quicko2 macOS UI.

Provides a sleek, macOS-native-feeling dark theme using Qt stylesheets.
"""

from PyQt6.QtWidgets import QApplication
from PyQt6.QtGui import QPalette, QColor
from PyQt6.QtCore import Qt

# Color palette
COLORS = {
    "bg_primary": "#0D0D0D",
    "bg_secondary": "#1A1A1A",
    "bg_tertiary": "#262626",
    "bg_hover": "#333333",
    "bg_input": "#1E1E1E",
    "text_primary": "#F5F5F5",
    "text_secondary": "#A0A0A0",
    "text_muted": "#666666",
    "accent": "#6C5CE7",
    "accent_hover": "#7C6CF7",
    "accent_pressed": "#5A4BD6",
    "success": "#00D68F",
    "warning": "#FFB800",
    "error": "#FF3D71",
    "border": "#2A2A2A",
    "border_focus": "#6C5CE7",
    "message_sent": "#2D2458",
    "message_received": "#1A1A2E",
    "scrollbar": "#3A3A3A",
    "scrollbar_hover": "#4A4A4A",
}


def apply_dark_theme(app: QApplication):
    """Apply the Quicko2 dark theme to the application."""
    
    # Set palette
    palette = QPalette()
    palette.setColor(QPalette.ColorRole.Window, QColor(COLORS["bg_primary"]))
    palette.setColor(QPalette.ColorRole.WindowText, QColor(COLORS["text_primary"]))
    palette.setColor(QPalette.ColorRole.Base, QColor(COLORS["bg_secondary"]))
    palette.setColor(QPalette.ColorRole.AlternateBase, QColor(COLORS["bg_tertiary"]))
    palette.setColor(QPalette.ColorRole.Text, QColor(COLORS["text_primary"]))
    palette.setColor(QPalette.ColorRole.Button, QColor(COLORS["bg_tertiary"]))
    palette.setColor(QPalette.ColorRole.ButtonText, QColor(COLORS["text_primary"]))
    palette.setColor(QPalette.ColorRole.Highlight, QColor(COLORS["accent"]))
    palette.setColor(QPalette.ColorRole.HighlightedText, QColor("#FFFFFF"))
    palette.setColor(QPalette.ColorRole.PlaceholderText, QColor(COLORS["text_muted"]))
    app.setPalette(palette)

    # Global stylesheet
    app.setStyleSheet(GLOBAL_STYLESHEET)


GLOBAL_STYLESHEET = f"""
/* ===== Global ===== */
QWidget {{
    background-color: {COLORS["bg_primary"]};
    color: {COLORS["text_primary"]};
    font-family: "SF Pro Display", "Helvetica Neue", sans-serif;
    selection-background-color: {COLORS["accent"]};
    selection-color: #FFFFFF;
}}

/* ===== Scrollbar ===== */
QScrollBar:vertical {{
    background: transparent;
    width: 8px;
    margin: 0;
}}
QScrollBar::handle:vertical {{
    background: {COLORS["scrollbar"]};
    border-radius: 4px;
    min-height: 30px;
}}
QScrollBar::handle:vertical:hover {{
    background: {COLORS["scrollbar_hover"]};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
    height: 0;
}}
QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{
    background: transparent;
}}

/* ===== Input Fields ===== */
QLineEdit, QTextEdit, QPlainTextEdit {{
    background-color: {COLORS["bg_input"]};
    border: 1px solid {COLORS["border"]};
    border-radius: 8px;
    padding: 8px 12px;
    color: {COLORS["text_primary"]};
    font-size: 14px;
}}
QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus {{
    border-color: {COLORS["border_focus"]};
}}

/* ===== Buttons ===== */
QPushButton {{
    background-color: {COLORS["accent"]};
    color: #FFFFFF;
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 600;
}}
QPushButton:hover {{
    background-color: {COLORS["accent_hover"]};
}}
QPushButton:pressed {{
    background-color: {COLORS["accent_pressed"]};
}}
QPushButton:disabled {{
    background-color: {COLORS["bg_tertiary"]};
    color: {COLORS["text_muted"]};
}}

/* ===== Secondary Button ===== */
QPushButton[cssClass="secondary"] {{
    background-color: {COLORS["bg_tertiary"]};
    color: {COLORS["text_primary"]};
    border: 1px solid {COLORS["border"]};
}}
QPushButton[cssClass="secondary"]:hover {{
    background-color: {COLORS["bg_hover"]};
}}

/* ===== Labels ===== */
QLabel {{
    background-color: transparent;
}}
QLabel[cssClass="title"] {{
    font-size: 24px;
    font-weight: 700;
    color: {COLORS["text_primary"]};
}}
QLabel[cssClass="subtitle"] {{
    font-size: 14px;
    color: {COLORS["text_secondary"]};
}}
QLabel[cssClass="muted"] {{
    font-size: 12px;
    color: {COLORS["text_muted"]};
}}

/* ===== Status Bar ===== */
QStatusBar {{
    background-color: {COLORS["bg_secondary"]};
    color: {COLORS["text_muted"]};
    font-size: 12px;
    border-top: 1px solid {COLORS["border"]};
}}

/* ===== Tool Tips ===== */
QToolTip {{
    background-color: {COLORS["bg_tertiary"]};
    color: {COLORS["text_primary"]};
    border: 1px solid {COLORS["border"]};
    border-radius: 4px;
    padding: 4px 8px;
}}
"""
