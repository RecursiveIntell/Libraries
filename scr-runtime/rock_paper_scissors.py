#!/usr/bin/env python3
"""
Rock Paper Scissors - A polished PySide6 game
"""

import sys
import random
from enum import Enum
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QLabel, QFrame, QGridLayout, QStackedWidget
)
from PySide6.QtCore import Qt, QPropertyAnimation, QEasingCurve, QTimer, Signal, QObject
from PySide6.QtGui import (
    QFont, QPalette, QColor, QPainter, QRadialGradient,
    QLinearGradient, QBrush, QPen, QIcon, QPixmap
)
from PySide6.QtSvgWidgets import QSvgWidget


class Choice(Enum):
    ROCK = "rock"
    PAPER = "paper"
    SCISSORS = "scissors"

    def beats(self, other):
        return {
            Choice.ROCK: Choice.SCISSORS,
            Choice.PAPER: Choice.ROCK,
            Choice.SCISSORS: Choice.PAPER
        }[self] == other

    def __str__(self):
        return self.value.capitalize()


class GameResult(Enum):
    WIN = "win"
    LOSE = "lose"
    TIE = "tie"


class AnimatedButton(QPushButton):
    def __init__(self, choice, parent=None):
        super().__init__(parent)
        self.choice = choice
        self.setFixedSize(140, 140)
        self.setCursor(Qt.PointingHandCursor)
        self.setStyleSheet("""
            QPushButton {
                border: none;
                background: transparent;
            }
            QPushButton:hover {
                background: rgba(255, 255, 255, 0.1);
            }
            QPushButton:pressed {
                background: rgba(255, 255, 255, 0.2);
            }
        """)

        self.svg_widget = QSvgWidget(self)
        self.svg_widget.setFixedSize(120, 120)
        self.svg_widget.move(10, 10)

        self.set_svg(choice)

    def set_svg(self, choice):
        svg_content = {
            Choice.ROCK: """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <defs>
                    <linearGradient id="rockGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:#6B7280"/>
                        <stop offset="100%" style="stop-color:#374151"/>
                    </linearGradient>
                </defs>
                <circle cx="50" cy="50" r="45" fill="url(#rockGrad)" stroke="#1F2937" stroke-width="3"/>
                <path d="M30 35 Q25 50 30 65 Q35 75 50 75 Q65 75 70 65 Q75 50 70 35 Q65 25 50 25 Q35 25 30 35" fill="#9CA3AF" stroke="#4B5563" stroke-width="2"/>
                <ellipse cx="40" cy="45" rx="5" ry="4" fill="#6B7280"/>
                <ellipse cx="60" cy="45" rx="5" ry="4" fill="#6B7280"/>
            </svg>""",
            Choice.PAPER: """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <defs>
                    <linearGradient id="paperGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:#F3F4F6"/>
                        <stop offset="100%" style="stop-color:#E5E7EB"/>
                    </linearGradient>
                </defs>
                <circle cx="50" cy="50" r="45" fill="url(#paperGrad)" stroke="#D1D5DB" stroke-width="3"/>
                <rect x="25" y="20" width="50" height="60" rx="3" fill="#FFFFFF" stroke="#9CA3AF" stroke-width="2"/>
                <line x1="30" y1="30" x2="70" y2="30" stroke="#D1D5DB" stroke-width="2"/>
                <line x1="30" y1="40" x2="70" y2="40" stroke="#D1D5DB" stroke-width="2"/>
                <line x1="30" y1="50" x2="70" y2="50" stroke="#D1D5DB" stroke-width="2"/>
                <line x1="30" y1="60" x2="55" y2="60" stroke="#D1D5DB" stroke-width="2"/>
            </svg>""",
            Choice.SCISSORS: """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <defs>
                    <linearGradient id="scissorsGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" style="stop-color:#EF4444"/>
                        <stop offset="100%" style="stop-color:#B91C1C"/>
                    </linearGradient>
                </defs>
                <circle cx="50" cy="50" r="45" fill="url(#scissorsGrad)" stroke="#7F1D1D" stroke-width="3"/>
                <ellipse cx="35" cy="35" rx="12" ry="15" fill="#FCA5A5" stroke="#DC2626" stroke-width="2" transform="rotate(-30 35 35)"/>
                <ellipse cx="65" cy="35" rx="12" ry="15" fill="#FCA5A5" stroke="#DC2626" stroke-width="2" transform="rotate(30 65 35)"/>
                <rect x="45" y="40" width="10" height="35" rx="2" fill="#9CA3AF" stroke="#6B7280" stroke-width="2"/>
                <circle cx="35" cy="35" r="5" fill="#7F1D1D"/>
                <circle cx="65" cy="35" r="5" fill="#7F1D1D"/>
            </svg>"""
        }[choice]

        self.svg_widget.load(svg_content.encode())


class ResultDisplay(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedSize(300, 200)
        self.result = None
        self.player_choice = None
        self.computer_choice = None
        self.animation_progress = 0

    def set_result(self, result, player_choice, computer_choice):
        self.result = result
        self.player_choice = player_choice
        self.computer_choice = computer_choice
        self.animation_progress = 0
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.Antialiasing)

        try:
            # Background gradient
            gradient = QLinearGradient(0, 0, self.width(), self.height())
            if self.result == GameResult.WIN:
                gradient.setColorAt(0, QColor(34, 197, 94))
                gradient.setColorAt(1, QColor(22, 163, 74))
            elif self.result == GameResult.LOSE:
                gradient.setColorAt(0, QColor(239, 68, 68))
                gradient.setColorAt(1, QColor(220, 38, 38))
            elif self.result == GameResult.TIE:
                gradient.setColorAt(0, QColor(251, 191, 36))
                gradient.setColorAt(1, QColor(245, 158, 11))
            else:
                # Default/empty state
                gradient.setColorAt(0, QColor(51, 65, 85))
                gradient.setColorAt(1, QColor(30, 41, 59))

            painter.setBrush(QBrush(gradient))
            painter.setPen(Qt.NoPen)
            painter.drawRoundedRect(self.rect(), 20, 20)

            # Result text
            if self.result is not None:
                painter.setPen(QColor(255, 255, 255))
                font = QFont("Arial", 32, QFont.Bold)
                painter.setFont(font)

                text = {
                    GameResult.WIN: "YOU WIN!",
                    GameResult.LOSE: "YOU LOSE!",
                    GameResult.TIE: "IT'S A TIE!"
                }[self.result]

                rect = self.rect()
                painter.drawText(rect, Qt.AlignCenter, text)
            else:
                # Empty state text
                painter.setPen(QColor(148, 163, 184))
                font = QFont("Arial", 20)
                painter.setFont(font)
                rect = self.rect()
                painter.drawText(rect, Qt.AlignCenter, "Choose your weapon!")
        finally:
            painter.end()


class ScoreBoard(QFrame):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFrameStyle(QFrame.StyledPanel)
        self.setStyleSheet("""
            QFrame {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:1,
                    stop:0 #1E293B, stop:1 #0F172A);
                border-radius: 15px;
                border: 2px solid #334155;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setSpacing(30)

        self.player_score = 0
        self.computer_score = 0
        self.ties = 0

        self.player_label = self.create_score_label("YOU", "#22C55E")
        self.computer_label = self.create_score_label("CPU", "#EF4444")
        self.tie_label = self.create_score_label("TIES", "#F59E0B")

        layout.addWidget(self.player_label)
        layout.addWidget(self.tie_label)
        layout.addWidget(self.computer_label)

    def create_score_label(self, title, color):
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setSpacing(5)

        title_label = QLabel(title)
        title_label.setAlignment(Qt.AlignCenter)
        title_label.setStyleSheet(f"color: {color}; font-size: 14px; font-weight: bold;")

        score_label = QLabel("0")
        score_label.setAlignment(Qt.AlignCenter)
        score_label.setStyleSheet(f"color: white; font-size: 36px; font-weight: bold;")
        score_label.setObjectName("score")

        layout.addWidget(title_label)
        layout.addWidget(score_label)

        return widget

    def update_score(self, result):
        if result == GameResult.WIN:
            self.player_score += 1
            self.player_label.findChild(QLabel, "score").setText(str(self.player_score))
        elif result == GameResult.LOSE:
            self.computer_score += 1
            self.computer_label.findChild(QLabel, "score").setText(str(self.computer_score))
        else:
            self.ties += 1
            self.tie_label.findChild(QLabel, "score").setText(str(self.ties))

    def reset(self):
        self.player_score = 0
        self.computer_score = 0
        self.ties = 0
        self.player_label.findChild(QLabel, "score").setText("0")
        self.computer_label.findChild(QLabel, "score").setText("0")
        self.tie_label.findChild(QLabel, "score").setText("0")


class RockPaperScissorsGame(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Rock Paper Scissors")
        self.setMinimumSize(800, 600)

        self.setup_ui()
        self.setup_animations()

    def setup_ui(self):
        central_widget = QWidget()
        self.setCentralWidget(central_widget)

        main_layout = QVBoxLayout(central_widget)
        main_layout.setSpacing(30)
        main_layout.setContentsMargins(40, 40, 40, 40)

        # Title
        title = QLabel("Rock Paper Scissors")
        title.setAlignment(Qt.AlignCenter)
        title.setStyleSheet("""
            QLabel {
                color: #F8FAFC;
                font-size: 48px;
                font-weight: bold;
            }
        """)
        main_layout.addWidget(title)

        # Score board
        self.score_board = ScoreBoard()
        main_layout.addWidget(self.score_board, 0, Qt.AlignCenter)

        # Result display
        self.result_display = ResultDisplay()
        main_layout.addWidget(self.result_display, 0, Qt.AlignCenter)

        # Choice buttons
        button_layout = QHBoxLayout()
        button_layout.setSpacing(40)

        self.rock_button = AnimatedButton(Choice.ROCK)
        self.paper_button = AnimatedButton(Choice.PAPER)
        self.scissors_button = AnimatedButton(Choice.SCISSORS)

        self.rock_button.clicked.connect(lambda: self.play_round(Choice.ROCK))
        self.paper_button.clicked.connect(lambda: self.play_round(Choice.PAPER))
        self.scissors_button.clicked.connect(lambda: self.play_round(Choice.SCISSORS))

        button_layout.addWidget(self.rock_button)
        button_layout.addWidget(self.paper_button)
        button_layout.addWidget(self.scissors_button)

        main_layout.addLayout(button_layout)

        # Reset button
        reset_button = QPushButton("Reset Score")
        reset_button.setCursor(Qt.PointingHandCursor)
        reset_button.setStyleSheet("""
            QPushButton {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                    stop:0 #6366F1, stop:1 #4F46E5);
                color: white;
                border: none;
                border-radius: 10px;
                padding: 12px 30px;
                font-size: 16px;
                font-weight: bold;
            }
            QPushButton:hover {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                    stop:0 #818CF8, stop:1 #6366F1);
            }
            QPushButton:pressed {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                    stop:0 #4F46E5, stop:1 #4338CA);
            }
        """)
        reset_button.clicked.connect(self.reset_game)
        main_layout.addWidget(reset_button, 0, Qt.AlignCenter)

        # Set window style
        self.setStyleSheet("""
            QMainWindow {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:1,
                    stop:0 #0F172A, stop:1 #1E293B);
            }
        """)

    def setup_animations(self):
        self.result_animation = QPropertyAnimation(self.result_display, b"geometry")
        self.result_animation.setDuration(300)
        self.result_animation.setEasingCurve(QEasingCurve.OutBack)

    def play_round(self, player_choice):
        computer_choice = random.choice(list(Choice))

        if player_choice == computer_choice:
            result = GameResult.TIE
        elif player_choice.beats(computer_choice):
            result = GameResult.WIN
        else:
            result = GameResult.LOSE

        self.result_display.set_result(result, player_choice, computer_choice)
        self.score_board.update_score(result)

        # Animate result
        self.result_animation.setStartValue(self.result_display.geometry())
        new_rect = self.result_display.geometry()
        new_rect.setHeight(220)
        self.result_animation.setEndValue(new_rect)
        self.result_animation.start()

    def reset_game(self):
        self.score_board.reset()
        self.result_display.result = None
        self.result_display.update()


def main():
    app = QApplication(sys.argv)

    # Set fusion style for consistent look
    app.setStyle("Fusion")

    # Set dark theme palette
    palette = QPalette()
    palette.setColor(QPalette.Window, QColor(15, 23, 42))
    palette.setColor(QPalette.WindowText, QColor(248, 250, 252))
    palette.setColor(QPalette.Base, QColor(30, 41, 59))
    palette.setColor(QPalette.AlternateBase, QColor(51, 65, 85))
    palette.setColor(QPalette.ToolTipBase, QColor(248, 250, 252))
    palette.setColor(QPalette.ToolTipText, QColor(15, 23, 42))
    palette.setColor(QPalette.Text, QColor(248, 250, 252))
    palette.setColor(QPalette.Button, QColor(51, 65, 85))
    palette.setColor(QPalette.ButtonText, QColor(248, 250, 252))
    palette.setColor(QPalette.BrightText, QColor(239, 68, 68))
    palette.setColor(QPalette.Link, QColor(99, 102, 241))
    palette.setColor(QPalette.Highlight, QColor(99, 102, 241))
    palette.setColor(QPalette.HighlightedText, QColor(255, 255, 255))
    app.setPalette(palette)

    game = RockPaperScissorsGame()
    game.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()