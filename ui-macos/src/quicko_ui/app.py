"""Quicko2 macOS Application — main entry point.

Launches the PyQt6 application with the dark theme and connects
to the Rust core via PyO3 bindings.
"""

import sys
from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import Qt

from .main_window import MainWindow
from .theme import apply_theme


def main():
    """Launch the Quicko2 application."""
    # High-DPI support for Apple Silicon
    app = QApplication(sys.argv)
    app.setApplicationName("Quicko2")
    app.setApplicationVersion("0.1.0")
    app.setOrganizationName("Quicko")

    # Apply dark theme (font is resolved inside apply_theme now)
    apply_theme(app)

    # Create and show main window
    window = MainWindow()
    window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
